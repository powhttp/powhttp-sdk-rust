use std::sync::Arc;
use serde::Deserialize;
use crate::runtime::handle::ExtensionHandle;
use crate::runtime::handlers::HandlerNotFound;
use crate::runtime::state::ExtensionState;
use crate::proxy_server::{ConnectContext, ConnectResult};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CallConnectHandlerParams {
    handler_id: String,
    context: ConnectContext,
}

pub(crate) async fn call_connect_handler(
    params: CallConnectHandlerParams,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<ConnectResult, tokio_jrpc::Error> {
    match state.get_connect_handler(&params.handler_id).await {
        Some(handler) => handler(params.context, ExtensionHandle::new(client, state)).await,
        None => Err(tokio_jrpc::Error::from(HandlerNotFound(params.handler_id))),
    }
}
