use std::future::Future;
use std::sync::Arc;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::Serialize;
use tokio_jrpc::ClientHandle;
use ulid::Ulid;
use crate::context_menu::{ContextMenuNodeMulti, ContextMenuNodeSingle};
use crate::error::Error;
use crate::proxy_server::{ConnectContext, ConnectResult};
use crate::http2::Http2Event;
use crate::overview::OverviewNode;
use crate::runtime::state::{ConnectHandler, ExtensionState};
use crate::sessions::{SessionEntry, SessionInfo};
use crate::tls::TlsEvent;
use crate::websocket::WebSocketMessage;

/// Handle used to communicate with powhttp.
///
/// Provides methods to register context-menu items, overview fields and connect
/// handlers, as well as querying session data. Cheaply cloneable.
#[derive(Clone)]
pub struct ExtensionHandle {
    client: ClientHandle,
    state: Arc<ExtensionState>,
}

impl ExtensionHandle {
    pub(crate) fn new(client: ClientHandle, state: Arc<ExtensionState>) -> Self {
        Self { client, state }
    }

    /// Gracefully shuts down the extension runtime.
    pub async fn shutdown(&self) {
        self.state.shutdown_handle().shutdown().await;
    }

    /// Writes a string to the system clipboard.
    ///
    /// Calls `clipboard/write_text`.
    pub async fn write_text_to_clipboard(&self, text: &str) -> Result<(), Error> {
        self.client
            .request("clipboard/write_text", ClipboardWriteTextParams { text }).await
            .map_err(Error::from)
    }

    /// Adds items to the single-entry context menu.
    ///
    /// Accepts anything that converts into a [`ContextMenuNodeSingle`], typically a
    /// [`ContextMenuItemSingle`](crate::ContextMenuItemSingle) or [`ContextMenuSubmenuSingle`](crate::ContextMenuSubmenuSingle).
    ///
    /// Calls `context_menu/extend_single`.
    pub async fn extend_context_menu_single<N: Into<ContextMenuNodeSingle>>(&self, node: N) -> Result<(), Error> {
        let node = node.into();
        let handler_pairs = node.extract_handlers();
        let handler_ids: Vec<String> = handler_pairs.iter().map(|(id, _)| id.clone()).collect();

        self.state.extend_context_menu_single_handlers(handler_pairs).await;

        if let Err(err) = self.client.request::<()>("context_menu/extend_single", &node).await {
            self.state.remove_context_menu_single_handler(&handler_ids).await;
            return Err(err.into());
        }
        Ok(())
    }

    /// Removes a single-entry context-menu item by its `item_id`.
    ///
    /// Calls `context_menu/remove_item_single`.
    pub async fn remove_context_menu_item_single(&self, item_id: &str) -> Result<(), Error> {
        self.client.request::<()>(
            "context_menu/remove_item_single",
            ContextMenuItemRef { item_id },
        ).await?;

        self.state.remove_context_menu_single_handler(&[item_id]).await;
        Ok(())
    }

    /// Adds items to the multi-entry context menu.
    ///
    /// Accepts anything that converts into a [`ContextMenuNodeMulti`], typically a
    /// [`ContextMenuItemMulti`](crate::ContextMenuItemMulti) or [`ContextMenuSubmenuMulti`](crate::ContextMenuSubmenuMulti).
    ///
    /// Calls `context_menu/extend_multi`.
    pub async fn extend_context_menu_multi<N: Into<ContextMenuNodeMulti>>(&self, node: N) -> Result<(), Error> {
        let node = node.into();
        let handler_pairs = node.extract_handlers();
        let handler_ids: Vec<String> = handler_pairs.iter().map(|(id, _)| id.clone()).collect();

        self.state.extend_context_menu_multi_handlers(handler_pairs).await;

        if let Err(err) = self.client.request::<()>("context_menu/extend_multi", &node).await {
            self.state.remove_context_menu_multi_handler(&handler_ids).await;
            return Err(err.into());
        }
        Ok(())
    }

    /// Removes a multi-entry context-menu item by its `item_id`.
    ///
    /// Calls `context_menu/remove_item_multi`.
    pub async fn remove_context_menu_item_multi(&self, item_id: &str) -> Result<(), Error> {
        self.client.request::<()>(
            "context_menu/remove_item_multi",
            ContextMenuItemRef { item_id },
        ).await?;

        self.state.remove_context_menu_multi_handler(&[item_id]).await;
        Ok(())
    }

    /// Adds fields or sections to the Overview section of the Inspector.
    ///
    /// Accepts anything that converts into an [`OverviewNode`], typically an
    /// [`OverviewField`](crate::OverviewField) or [`OverviewSection`](crate::OverviewSection).
    ///
    /// Calls `overview/extend`.
    pub async fn extend_overview<N: Into<OverviewNode>>(&self, node: N) -> Result<(), Error> {
        let node = node.into();
        let handler_pairs = node.extract_handlers();
        let handler_ids: Vec<String> = handler_pairs.iter().map(|(id, _)| id.clone()).collect();

        self.state.extend_overview_handlers(handler_pairs).await;

        if let Err(err) = self.client.request::<()>("overview/extend", &node).await {
            self.state.remove_overview_handler(&handler_ids).await;
            return Err(err.into());
        }
        Ok(())
    }

    /// Removes an overview field by its `field_id`.
    ///
    /// Calls `overview/remove_field`.
    pub async fn remove_overview_field(&self, field_id: &str) -> Result<(), Error> {
        self.client.request::<()>(
            "overview/remove_field",
            OverviewFieldRef { field_id }
        ).await?;

        self.state.remove_overview_handler(&[field_id]).await;
        Ok(())
    }

    /// Registers a handler that is invoked for every new proxy connection.
    ///
    /// The handler receives a [`ConnectContext`] describing the incoming connection
    /// and must return a [`ConnectResult`] to accept or reject it.
    ///
    /// Calls `proxy_server/add_connect_handler`.
    pub async fn add_connect_handler<F, Fut>(&self, handler_id: &str, handler: F) -> Result<(), Error>
    where
        F: Fn(ConnectContext, ExtensionHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ConnectResult, Error>> + Send + 'static,
    {
        let id = handler_id.to_owned();
        let handler: ConnectHandler = Arc::new(move |ctx, handle| {
            let fut = handler(ctx, handle);
            Box::pin(async move { fut.await.map_err(Error::into_jrpc) })
        });

        self.state.insert_connect_handlers(id.clone(), handler).await;

        let add_result = self.client.request::<()>(
            "proxy_server/add_connect_handler",
            ConnectHandlerRef { handler_id }
        ).await;

        if let Err(err) = add_result {
            self.state.remove_connect_handler(&id).await;
            return Err(err.into());
        }
        Ok(())
    }

    /// Unregisters a previously added connect handler.
    ///
    /// Calls `proxy_server/remove_connect_handler`.
    pub async fn remove_connect_handler(&self, handler_id: &str) -> Result<(), Error> {
        self.client.request::<()>(
            "proxy_server/remove_connect_handler",
            ConnectHandlerRef { handler_id },
        ).await?;
        
        self.state.remove_connect_handler(&handler_id).await;
        Ok(())
    }

    /// Returns all open sessions.
    ///
    /// Calls `sessions/list`.
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>, Error> {
        self.client
            .request("sessions/list", ()).await
            .map_err(Error::from)
    }

    /// Returns the local port the powhttp proxy server is listening on for the given session.
    ///
    /// Calls `sessions/get_listener_port`.
    pub async fn get_listener_port(&self, session_id: Ulid) -> Result<Option<u16>, Error> {
        self.client
            .request("sessions/get_listener_port", SessionRef { session_id }).await
            .map_err(Error::from)
    }

    /// Returns the entry IDs within a session.
    ///
    /// Calls `sessions/get_entry_ids`.
    pub async fn get_session_entry_ids(&self, session_id: Ulid) -> Result<Option<Vec<Ulid>>, Error> {
        self.client
            .request("sessions/get_entry_ids", SessionRef { session_id }).await
            .map_err(Error::from)
    }

    /// Fetches a full session entry including request, response and timings.
    ///
    /// Calls `sessions/get_entry`.
    pub async fn get_session_entry(&self, session_id: Ulid, entry_id: Ulid) -> Result<Option<SessionEntry>, Error> {
        self.client
            .request("sessions/get_entry", SessionEntryRef { session_id, entry_id }).await
            .map_err(Error::from)
    }

    /// Returns the request body decoded as a UTF-8 string.
    ///
    /// Calls `sessions/get_request_body`.
    pub async fn get_request_body_as_text(&self, session_id: Ulid, entry_id: Ulid) -> Result<Option<String>, Error> {
        self.client
            .request(
                "sessions/get_request_body", 
                GetBodyParams { session_id, entry_id, encoding: BodyEncoding::Text }
            )
            .await
            .map_err(Error::from)
    }

    /// Returns the request body as raw bytes.
    ///
    /// Calls `sessions/get_request_body`.
    pub async fn get_request_body_as_bytes(&self, session_id: Ulid, entry_id: Ulid) -> Result<Option<Vec<u8>>, Error> {
        self.client
            .request::<Option<String>>(
                "sessions/get_request_body",
                GetBodyParams { session_id, entry_id, encoding: BodyEncoding::Base64 },
            )
            .await?
            .map(|body_text| BASE64_STANDARD.decode(body_text).map_err(Error::new))
            .transpose()
    }

    /// Returns the response body decoded as a UTF-8 string.
    ///
    /// Calls `sessions/get_response_body`.
    pub async fn get_response_body_as_text(&self, session_id: Ulid, entry_id: Ulid) -> Result<Option<String>, Error> {
        self.client
            .request(
                "sessions/get_response_body",
                GetBodyParams { session_id, entry_id, encoding: BodyEncoding::Text },
            )
            .await
            .map_err(Error::from)
    }

    /// Returns the response body as raw bytes.
    ///
    /// Calls `sessions/get_response_body`.
    pub async fn get_response_body_as_bytes(&self, session_id: Ulid, entry_id: Ulid) -> Result<Option<Vec<u8>>, Error> {
        self.client
            .request::<Option<String>>(
                "sessions/get_response_body",
                GetBodyParams { session_id, entry_id, encoding: BodyEncoding::Base64 },
            )
            .await?
            .map(|body_text| BASE64_STANDARD.decode(body_text).map_err(Error::new))
            .transpose()
    }

    /// Returns all WebSocket messages for a session entry.
    ///
    /// Calls `sessions/get_websocket_messages`.
    pub async fn get_websocket_messages(&self, session_id: Ulid, entry_id: Ulid) -> Result<Option<Vec<WebSocketMessage>>, Error> {
        self.client
            .request(
                "sessions/get_websocket_messages",
                SessionEntryRef { session_id, entry_id },
            )
            .await
            .map_err(Error::from)
    }

    /// Returns the TLS events for a connection.
    ///
    /// Calls `tls/get_connection`.
    pub async fn get_tls_connection(&self, connection_id: Ulid) -> Result<Option<Vec<TlsEvent>>, Error> {
        self.client
            .request("tls/get_connection", TlsConnectionRef { connection_id }).await
            .map_err(Error::from)
    }

    /// Returns the HTTP/2 stream IDs for a connection.
    ///
    /// Calls `http2/get_stream_ids`.
    pub async fn get_http2_stream_ids(&self, connection_id: Ulid) -> Result<Option<Vec<u32>>, Error> {
        self.client
            .request("http2/get_stream_ids", Http2ConnectionRef { connection_id }).await
            .map_err(Error::from)
    }

    /// Returns the HTTP/2 frames for a specific stream.
    ///
    /// Calls `http2/get_stream`.
    pub async fn get_http2_stream(&self, connection_id: Ulid, stream_id: u32) -> Result<Option<Vec<Http2Event>>, Error> {
        self.client
            .request("http2/get_stream", Http2StreamRef { connection_id, stream_id }).await
            .map_err(Error::from)
    }
}

#[derive(Serialize)]
struct ClipboardWriteTextParams<'a> {
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextMenuItemRef<'a> {
    item_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverviewFieldRef<'a> {
    field_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectHandlerRef<'a> {
    handler_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionRef {
    session_id: Ulid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionEntryRef {
    session_id: Ulid,
    entry_id: Ulid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetBodyParams {
    session_id: Ulid,
    entry_id: Ulid,
    encoding: BodyEncoding,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyEncoding {
    Text,
    #[serde(rename = "base64")]
    Base64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TlsConnectionRef {
    connection_id: Ulid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Http2ConnectionRef {
    connection_id: Ulid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Http2StreamRef {
    pub connection_id: Ulid,
    pub stream_id: u32,
}