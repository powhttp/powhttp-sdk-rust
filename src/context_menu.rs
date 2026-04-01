use std::sync::Arc;
use serde::Serialize;

use crate::runtime::handlers::{MultiEntryContext, SingleEntryContext};
use crate::runtime::state::ContextMenuHandler;
use crate::runtime::handle::ExtensionHandle;
use crate::error::Error;

/// A clickable item in the session context menu.
///
/// The type parameter `C` determines whether this item operates on a single
/// entry ([`SingleEntryContext`]) or multiple entries ([`MultiEntryContext`]).
///
/// ```
/// use powhttp_sdk::{ContextMenuItemSingle, ExtensionHandle, SingleEntryContext, Error};
///
/// let item = ContextMenuItemSingle::new(
///     "copy-url",
///     "Copy URL",
///     async |ctx: SingleEntryContext, handle: ExtensionHandle| {
///         // handler logic here
///         Ok(())
///     },
/// );
/// ```
#[derive(Serialize)]
#[serde(bound(serialize = ""))]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuItem<C> {
    id: String,
    label: String,
    #[serde(skip)]
    handler: ContextMenuHandler<C>,
}

impl<C> ContextMenuItem<C> {
    /// Creates a new context-menu item with the given `id`, display `label` and async handler.
    pub fn new<F, Fut>(id: impl Into<String>, label: impl Into<String>, handler: F) -> Self
    where
        F: Fn(C, ExtensionHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static,
    {
        Self {
            id: id.into(),
            label: label.into(),
            handler: Arc::new(move |ctx, handle| {
                let fut = handler(ctx, handle);
                Box::pin(async move { fut.await.map_err(Error::into_jrpc) })
            }),
        }
    }
}

/// A submenu that groups context-menu items under a shared label.
#[derive(Serialize)]
#[serde(bound(serialize = ""))]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuSubmenu<C> {
    label: String,
    children: Vec<ContextMenuNode<C>>,
}

impl<C> ContextMenuSubmenu<C> {
    /// Creates an empty submenu with the given display `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new()
        }
    }

    /// Appends all nodes from `children`, draining the provided vec.
    pub fn with_children(mut self, children: &mut Vec<ContextMenuNode<C>>) -> Self {
        self.children.append(children);
        self
    }

    /// Appends a single child node (item or nested submenu).
    pub fn with_child(mut self, child: ContextMenuNode<C>) -> Self {
        self.children.push(child);
        self
    }
}

/// A node in the context-menu tree, either a leaf [`Item`](ContextMenuNode::Item)
/// or a [`Submenu`](ContextMenuNode::Submenu).
#[derive(Serialize)]
#[serde(bound(serialize = ""))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextMenuNode<C> {
    Submenu(ContextMenuSubmenu<C>),
    Item(ContextMenuItem<C>),
}

impl<C> ContextMenuNode<C> {
    pub(crate) fn extract_handlers(&self) -> Vec<(String, ContextMenuHandler<C>)> {
        match self {
            Self::Item(item) => vec![(item.id.clone(), Arc::clone(&item.handler))],
            Self::Submenu(sub) => sub
                .children
                .iter()
                .flat_map(|child| child.extract_handlers())
                .collect(),
        }
    }
}

impl<C> From<ContextMenuItem<C>> for ContextMenuNode<C> {
    fn from(item: ContextMenuItem<C>) -> Self {
        ContextMenuNode::Item(item)
    }
}

impl<C> From<ContextMenuSubmenu<C>> for ContextMenuNode<C> {
    fn from(submenu: ContextMenuSubmenu<C>) -> Self {
        ContextMenuNode::Submenu(submenu)
    }
}

/// Context-menu item for single-entry selections.
pub type ContextMenuItemSingle = ContextMenuItem<SingleEntryContext>;
/// Context-menu submenu for single-entry selections.
pub type ContextMenuSubmenuSingle = ContextMenuSubmenu<SingleEntryContext>;
/// Context-menu node for single-entry selections.
pub type ContextMenuNodeSingle = ContextMenuNode<SingleEntryContext>;

/// Context-menu item for multi-entry selections.
pub type ContextMenuItemMulti = ContextMenuItem<MultiEntryContext>;
/// Context-menu submenu for multi-entry selections.
pub type ContextMenuSubmenuMulti = ContextMenuSubmenu<MultiEntryContext>;
/// Context-menu node for multi-entry selections.
pub type ContextMenuNodeMulti = ContextMenuNode<MultiEntryContext>;
