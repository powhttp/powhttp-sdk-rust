use std::sync::Arc;
use serde::Deserialize;
use ulid::Ulid;
use crate::runtime::handlers::{SingleEntryContext, HandlerNotFound};
use crate::runtime::state::ExtensionState;
use crate::runtime::handle::ExtensionHandle;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetOverviewFieldValueParams {
    field_id: String,
    session_id: Ulid,
    entry_id: Ulid,
}

pub(crate) async fn get_field_value(
    params: GetOverviewFieldValueParams,
    client: tokio_jrpc::ClientHandle,
    state: Arc<ExtensionState>,
) -> Result<Option<String>, tokio_jrpc::Error> {
    match state.get_overview_handler(&params.field_id).await {
        Some(handler) => {
            let ctx = SingleEntryContext {
                session_id: params.session_id,
                entry_id: params.entry_id,
            };
            handler(ctx, ExtensionHandle::new(client, state)).await
        }
        None => Err(tokio_jrpc::Error::from(HandlerNotFound(params.field_id))),
    }
}