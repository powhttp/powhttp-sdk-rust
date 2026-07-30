use std::sync::Arc;
use serde::{Serialize, Serializer};

use crate::runtime::handle::ExtensionHandle;
use crate::runtime::handlers::SingleEntryContext;
use crate::runtime::state::{MessageTabContentHandler, MessageTabHandlers, MessageTabVisibilityHandler};
use crate::error::Error;

/// Syntax highlighting applied to the text of a [`TabContent`].
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Text,
    Json,
    Xml,
    Html,
    Css,
    JavaScript,
    Yaml,
}

/// The contents rendered inside a [`MessageTab`].
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TabContent {
    text: String,
    language: Language,
}

impl TabContent {
    /// Creates contents with an explicit `language`.
    pub fn new(text: impl Into<String>, language: Language) -> Self {
        Self {
            text: text.into(),
            language,
        }
    }

    /// Creates contents without syntax highlighting.
    pub fn plain(text: impl Into<String>) -> Self {
        Self::new(text, Language::Text)
    }

    /// Creates contents highlighted as JSON.
    pub fn json(text: impl Into<String>) -> Self {
        Self::new(text, Language::Json)
    }

    /// Creates contents highlighted as XML.
    pub fn xml(text: impl Into<String>) -> Self {
        Self::new(text, Language::Xml)
    }

    /// Creates contents highlighted as HTML.
    pub fn html(text: impl Into<String>) -> Self {
        Self::new(text, Language::Html)
    }

    /// Creates contents highlighted as CSS.
    pub fn css(text: impl Into<String>) -> Self {
        Self::new(text, Language::Css)
    }

    /// Creates contents highlighted as JavaScript.
    pub fn javascript(text: impl Into<String>) -> Self {
        Self::new(text, Language::JavaScript)
    }

    /// Creates contents highlighted as YAML.
    pub fn yaml(text: impl Into<String>) -> Self {
        Self::new(text, Language::Yaml)
    }
}

/// A tab displayed next to Headers, Cookies, Query, Body and Raw in the request
/// or response section of the Inspector.
///
/// The content handler receives a [`SingleEntryContext`] and returns the
/// [`TabContent`] to render. A tab is visible for every entry unless
/// [`visible_when`](MessageTab::visible_when) narrows it down.
///
/// ```
/// use base64::prelude::{BASE64_URL_SAFE_NO_PAD, Engine};
/// use powhttp_sdk::{MessageTab, TabContent, ExtensionHandle, SingleEntryContext};
/// use powhttp_sdk::sessions::SessionEntry;
///
/// fn bearer_token(entry: &SessionEntry) -> Option<&str> {
///     entry.request.headers.get("authorization")?.strip_prefix("Bearer ")
/// }
///
/// fn decode_jwt_payload(token: &str) -> Option<String> {
///     let payload = token.split('.').nth(1)?;
///     let bytes = BASE64_URL_SAFE_NO_PAD.decode(payload).ok()?;
///     String::from_utf8(bytes).ok()
/// }
///
/// let tab = MessageTab::new(
///     "jwt",
///     "JWT",
///     async |ctx: SingleEntryContext, handle: ExtensionHandle| {
///         let entry = handle.get_session_entry(ctx.session_id, ctx.entry_id).await?;
///         let payload = entry
///             .as_ref()
///             .and_then(bearer_token)
///             .and_then(decode_jwt_payload)
///             .unwrap_or_default();
///         Ok(TabContent::json(payload))
///     },
/// )
/// .visible_when(async |ctx: SingleEntryContext, handle: ExtensionHandle| {
///     let entry = handle.get_session_entry(ctx.session_id, ctx.entry_id).await?;
///     Ok(entry.as_ref().and_then(bearer_token).is_some())
/// });
/// ```
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageTab {
    id: String,
    label: String,
    #[serde(skip)]
    content: MessageTabContentHandler,
    #[serde(rename = "isConditional", serialize_with = "serialize_is_some")]
    visibility: Option<MessageTabVisibilityHandler>,
}

impl MessageTab {
    /// Creates a new tab with the given `id`, display `label` and async content handler.
    pub fn new<F, Fut>(id: impl Into<String>, label: impl Into<String>, content: F) -> Self
    where
        F: Fn(SingleEntryContext, ExtensionHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<TabContent, Error>> + Send + 'static,
    {
        Self {
            id: id.into(),
            label: label.into(),
            content: Arc::new(move |ctx, handle| {
                let fut = content(ctx, handle);
                Box::pin(async move { fut.await.map_err(Error::into_jrpc) })
            }),
            visibility: None,
        }
    }

    /// Restricts the tab to entries for which `visibility` resolves to `true`.
    ///
    /// The predicate runs whenever an entry is selected, so it should avoid
    /// expensive work such as fetching bodies.
    pub fn visible_when<F, Fut>(mut self, visibility: F) -> Self
    where
        F: Fn(SingleEntryContext, ExtensionHandle) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<bool, Error>> + Send + 'static,
    {
        self.visibility = Some(Arc::new(move |ctx, handle| {
            let fut = visibility(ctx, handle);
            Box::pin(async move { fut.await.map_err(Error::into_jrpc) })
        }));
        self
    }

    pub(crate) fn extract_handlers(&self) -> (String, MessageTabHandlers) {
        (
            self.id.clone(),
            MessageTabHandlers {
                content: Arc::clone(&self.content),
                visibility: self.visibility.as_ref().map(Arc::clone),
            },
        )
    }
}

fn serialize_is_some<S: Serializer>(
    visibility: &Option<MessageTabVisibilityHandler>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_bool(visibility.is_some())
}
