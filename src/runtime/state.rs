use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::{Mutex, RwLock, oneshot};
use crate::proxy_server::{ConnectContext, ConnectResult};
use crate::runtime::handlers::{MultiEntryContext, SingleEntryContext};
use crate::runtime::handle::ExtensionHandle;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
type Handler<P, R> = Arc<dyn Fn(P, ExtensionHandle) -> BoxFuture<Result<R, tokio_jrpc::Error>> + Send + Sync>;

pub(crate) type ContextMenuHandler<C> = Handler<C, ()>;
pub(crate) type OverviewHandler = Handler<SingleEntryContext, Option<String>>;
pub(crate) type ConnectHandler = Handler<ConnectContext, ConnectResult>;

#[derive(Clone)]
pub(crate) struct ShutdownHandle {
    tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl ShutdownHandle {
    pub(crate) fn new(tx: oneshot::Sender<()>) -> Self {
        Self {
            tx: Arc::new(Mutex::new(Some(tx))),
        }
    }

    pub(crate) async fn shutdown(&self) {
        if let Some(tx) = self.tx.lock().await.take() {
            let _ = tx.send(());
        }
    }
}

pub(crate) struct ExtensionState {
    shutdown_handle: ShutdownHandle,
    context_menu_single_handlers: RwLock<HashMap<String, ContextMenuHandler<SingleEntryContext>>>,
    context_menu_multi_handlers: RwLock<HashMap<String, ContextMenuHandler<MultiEntryContext>>>,
    overview_handlers: RwLock<HashMap<String, OverviewHandler>>,
    connect_handlers: RwLock<HashMap<String, ConnectHandler>>,
}

impl ExtensionState {
    pub(crate) fn new(shutdown: ShutdownHandle) -> Self {
        Self {
            shutdown_handle: shutdown,
            context_menu_single_handlers: RwLock::new(HashMap::new()),
            context_menu_multi_handlers: RwLock::new(HashMap::new()),
            overview_handlers: RwLock::new(HashMap::new()),
            connect_handlers: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn shutdown_handle(&self) -> &ShutdownHandle {
        &self.shutdown_handle
    }

    pub(crate) async fn extend_context_menu_single_handlers<H>(&self, handler_pairs: H)
    where H: IntoIterator<Item = (String, ContextMenuHandler<SingleEntryContext>)> 
    {
        self.context_menu_single_handlers
            .write()
            .await
            .extend(handler_pairs);
    }

    pub(crate) async fn extend_context_menu_multi_handlers<H>(&self, handler_pairs: H)
    where H: IntoIterator<Item = (String, ContextMenuHandler<MultiEntryContext>)> 
    {
        self.context_menu_multi_handlers
            .write()
            .await
            .extend(handler_pairs);
    }

    pub(crate) async fn extend_overview_handlers<H>(&self, handler_pairs: H)
    where H: IntoIterator<Item = (String, OverviewHandler)> 
    {
        self.overview_handlers
            .write()
            .await
            .extend(handler_pairs);
    }

    pub(crate) async fn insert_connect_handlers(&self, handler_id: String, handler: ConnectHandler) {
        self.connect_handlers.write().await.insert(handler_id, handler);
    }

    pub(crate) async fn remove_context_menu_single_handler(&self, handler_ids: &[impl AsRef<str>]) {
        let mut handlers = self.context_menu_single_handlers.write().await;
        for id in handler_ids {
            handlers.remove(id.as_ref());
        }
    }

    pub(crate) async fn remove_context_menu_multi_handler(&self, handler_ids: &[impl AsRef<str>]) {
        let mut handlers = self.context_menu_multi_handlers.write().await;
        for id in handler_ids {
            handlers.remove(id.as_ref());
        }
    }

    pub(crate) async fn remove_overview_handler(&self, handler_ids: &[impl AsRef<str>]) {
        let mut handlers = self.overview_handlers.write().await;
        for id in handler_ids {
            handlers.remove(id.as_ref());
        }
    }

    pub(crate) async fn remove_connect_handler(&self, handler_id: &str) {
        self.connect_handlers.write().await.remove(handler_id);
    }

    pub(crate) async fn get_context_menu_single_handler(&self, handler_id: &str) -> Option<ContextMenuHandler<SingleEntryContext>> {
        self.context_menu_single_handlers
            .read()
            .await
            .get(handler_id)
            .cloned()
    }

    pub(crate) async fn get_context_menu_multi_handler(&self, handler_id: &str) -> Option<ContextMenuHandler<MultiEntryContext>> {
        self.context_menu_multi_handlers
            .read()
            .await
            .get(handler_id)
            .cloned()
    }

    pub(crate) async fn get_overview_handler(&self, handler_id: &str) -> Option<OverviewHandler> {
        self.overview_handlers
            .read()
            .await
            .get(handler_id)
            .cloned()
    }

    pub(crate) async fn get_connect_handler(&self, handler_id: &str) -> Option<ConnectHandler> {
        self.connect_handlers
            .read()
            .await
            .get(handler_id)
            .cloned()
    }
}
