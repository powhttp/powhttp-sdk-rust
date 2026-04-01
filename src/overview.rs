use std::sync::Arc;
use serde::Serialize;

use crate::runtime::handle::ExtensionHandle;
use crate::runtime::state::OverviewHandler;
use crate::runtime::handlers::SingleEntryContext;
use crate::error::Error;

/// A single computed field displayed in the Overview section of the Inspector.
///
/// The handler receives a [`SingleEntryContext`] and returns an optional string
/// value to display. Return `None` to hide the field for that entry.
///
/// ```
/// use powhttp_sdk::{OverviewField, ExtensionHandle, SingleEntryContext, Error};
///
/// let field = OverviewField::new(
///     "content-length",
///     "Content Length",
///     async |ctx: SingleEntryContext, handle: ExtensionHandle| {
///         let entry = handle.get_session_entry(ctx.session_id, ctx.entry_id).await?;
///         let size = entry
///             .and_then(|entry| entry.response)
///             .and_then(|res| res.body_size)
///             .map(|size| size.to_string());
///         Ok(size)
///     },
/// );
/// ```
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewField {
    id: String,
    label: String,
    #[serde(skip)]
    handler: OverviewHandler,
}

impl OverviewField {
    /// Creates a new overview field with the given `id`, display `label` and async handler.
    pub fn new<F, Fut>(id: impl Into<String>, label: impl Into<String>, handler: F) -> Self
    where
        F: Fn(SingleEntryContext, ExtensionHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<String>, Error>> + Send + 'static,
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

/// A named group of overview fields, displayed as a collapsible section.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewSection {
    label: String,
    children: Vec<OverviewNode>,
}

impl OverviewSection {
    /// Creates an empty section with the given display `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new()
        }
    }

    /// Appends all nodes from `children`, draining the provided vec.
    pub fn with_children(mut self, children: &mut Vec<OverviewNode>) -> Self {
        self.children.append(children);
        self
    }

    /// Appends a single child node (field or nested section).
    pub fn with_child(mut self, child: OverviewNode) -> Self {
        self.children.push(child);
        self
    }
}

/// A node in the overview tree, either a leaf [`Field`](OverviewNode::Field)
/// or a [`Section`](OverviewNode::Section).
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OverviewNode {
    Section(OverviewSection),
    Field(OverviewField),
}

impl From<OverviewField> for OverviewNode {
    fn from(field: OverviewField) -> Self {
        OverviewNode::Field(field)
    }
}

impl From<OverviewSection> for OverviewNode {
    fn from(section: OverviewSection) -> Self {
        OverviewNode::Section(section)
    }
}

impl OverviewNode {
    pub(crate) fn extract_handlers(&self) -> Vec<(String, OverviewHandler)> {
        match self {
            Self::Field(field) => vec![(field.id.clone(), Arc::clone(&field.handler))],
            Self::Section(section) => section
                .children
                .iter()
                .flat_map(|child| child.extract_handlers())
                .collect(),
        }
    }
}