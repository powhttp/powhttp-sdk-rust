use std::sync::Arc;
use serde::Deserialize;
use ulid::Ulid;
use crate::inspector::TabContent;
use crate::runtime::handle::ExtensionHandle;
use crate::runtime::handlers::{HandlerNotFound, SingleEntryContext};
use crate::runtime::state::{ExtensionState, MessageTabHandlers};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageTabRef {
    tab_id: String,
    session_id: Ulid,
    entry_id: Ulid,
}

pub(crate) async fn is_request_tab_visible(
    params: MessageTabRef,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<bool, tokio_jrpc::Error> {
    let handlers = state.get_request_tab_handlers(&params.tab_id).await;
    is_tab_visible(params, client, state, handlers).await
}

pub(crate) async fn is_response_tab_visible(
    params: MessageTabRef,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<bool, tokio_jrpc::Error> {
    let handlers = state.get_response_tab_handlers(&params.tab_id).await;
    is_tab_visible(params, client, state, handlers).await
}

pub(crate) async fn get_request_tab_content(
    params: MessageTabRef,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<TabContent, tokio_jrpc::Error> {
    let handlers = state.get_request_tab_handlers(&params.tab_id).await;
    get_tab_content(params, client, state, handlers).await
}

pub(crate) async fn get_response_tab_content(
    params: MessageTabRef,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<TabContent, tokio_jrpc::Error> {
    let handlers = state.get_response_tab_handlers(&params.tab_id).await;
    get_tab_content(params, client, state, handlers).await
}

async fn is_tab_visible(
    params: MessageTabRef,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
    handlers: Option<MessageTabHandlers>,
) -> Result<bool, tokio_jrpc::Error> {
    let Some(handlers) = handlers else {
        return Err(tokio_jrpc::Error::from(HandlerNotFound(params.tab_id)));
    };

    match handlers.visibility {
        Some(visibility) => {
            let ctx = SingleEntryContext {
                session_id: params.session_id,
                entry_id: params.entry_id,
            };
            visibility(ctx, ExtensionHandle::new(client, state)).await
        }
        None => Ok(true),
    }
}

async fn get_tab_content(
    params: MessageTabRef,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
    handlers: Option<MessageTabHandlers>,
) -> Result<TabContent, tokio_jrpc::Error> {
    let Some(handlers) = handlers else {
        return Err(tokio_jrpc::Error::from(HandlerNotFound(params.tab_id)));
    };

    let ctx = SingleEntryContext {
        session_id: params.session_id,
        entry_id: params.entry_id,
    };
    (handlers.content)(ctx, ExtensionHandle::new(client, state)).await
}
