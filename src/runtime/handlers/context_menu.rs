use std::sync::Arc;
use serde::Deserialize;
use ulid::Ulid;
use crate::runtime::handle::ExtensionHandle;
use crate::runtime::handlers::{MultiEntryContext, SingleEntryContext, HandlerNotFound};
use crate::runtime::state::ExtensionState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CallItemHandlerSingleParams {
    item_id: String,
    session_id: Ulid,
    entry_id: Ulid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CallItemHandlerMultiParams {
    item_id: String,
    session_id: Ulid,
    entry_ids: Vec<Ulid>,
}

pub(crate) async fn call_item_handler_single(
    params: CallItemHandlerSingleParams,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<(), tokio_jrpc::Error> {
    match state.get_context_menu_single_handler(&params.item_id).await {
        Some(handler) => {
            let ctx = SingleEntryContext {
                session_id: params.session_id,
                entry_id: params.entry_id,
            };
            handler(ctx, ExtensionHandle::new(client, state)).await
        }
        None => Err(tokio_jrpc::Error::from(HandlerNotFound(params.item_id))),
    }
}

pub(crate) async fn call_item_handler_multi(
    params: CallItemHandlerMultiParams,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<(), tokio_jrpc::Error> {
    match state.get_context_menu_multi_handler(&params.item_id).await {
        Some(handler) => {
            let ctx = MultiEntryContext {
                session_id: params.session_id,
                entry_ids: params.entry_ids,
            };
            handler(ctx, ExtensionHandle::new(client, state)).await
        }
        None => Err(tokio_jrpc::Error::from(HandlerNotFound(params.item_id))),
    }
}