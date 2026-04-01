use std::future::Future;
use std::sync::Arc;
use crate::error::Error;
use crate::runtime::handle::ExtensionHandle;
use crate::runtime::state::{ExtensionState, ShutdownHandle};

pub(crate) mod handlers;
pub(crate) mod handle;
pub(crate) mod state;

/// Starts the extension runtime, calling `init` with an [`ExtensionHandle`].
///
/// This is the main entry point for an extension. The runtime runs until the
/// host disconnects or [`ExtensionHandle::shutdown`] is called.
///
/// ```rust,no_run
/// use powhttp_sdk::{run, Error, ExtensionHandle};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     run(async |handle: ExtensionHandle| {
///         Ok(())
///     }).await
/// }
/// ```
pub async fn run<F, Fut>(init: F) -> Result<(), Error>
where
    F: FnOnce(ExtensionHandle) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), Error>> + Send + 'static,
{
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let state = Arc::new(ExtensionState::new(ShutdownHandle::new(shutdown_tx)));

    let app = tokio_jrpc::App::new(tokio::io::stdin(), tokio::io::stdout(), state.clone())
        .method("context_menu/call_item_handler_single", handlers::context_menu::call_item_handler_single)
        .method("context_menu/call_item_handler_multi", handlers::context_menu::call_item_handler_multi)
        .method("overview/get_field_value", handlers::overview::get_field_value)
        .method("proxy_server/call_connect_handler", handlers::proxy_server::call_connect_handler);

    let client = app.client_handle();
    let handle = ExtensionHandle::new(client, state);

    let init_task = tokio::spawn(async move {
        let stop = handle.clone();
        match init(handle).await {
            Ok(()) => Ok(()),
            Err(err) => {
                eprintln!("extension init error: {err}");
                stop.shutdown().await;
                Err(err)
            }
        }
    });

    enum AppResult {
        Ran(Result<(), tokio_jrpc::RuntimeError>),
        Shutdown,
    }

    let run_fut = app.run();
    tokio::pin!(run_fut);

    let app_result = tokio::select! {
        r = &mut run_fut => AppResult::Ran(r),
        _ = shutdown_rx => AppResult::Shutdown,
    };

    init_task.await.map_err(Error::new)??;

    match app_result {
        AppResult::Ran(r) => r.map_err(Error::from),
        AppResult::Shutdown => Ok(()),
    }
}