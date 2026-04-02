# powhttp-sdk

[![docs.rs](https://img.shields.io/docsrs/powhttp-sdk?logo=rust)](https://docs.rs/powhttp-sdk)
[![crates.io](https://img.shields.io/crates/v/powhttp-sdk?logo=rust)](https://crates.io/crates/powhttp-sdk)

Official SDK for building [powhttp](https://powhttp.com/) extensions in Rust.

Extensions run as separate processes that communicate with powhttp over JSON-RPC.
The SDK handles the runtime lifecycle and provides an `ExtensionHandle` for
registering context-menu items, overview fields, connect handlers and querying
session data.

## Example

```rust
use powhttp_sdk::{
    run, Error, ExtensionHandle,
    ContextMenuItemSingle, SingleEntryContext,
    OverviewField,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(async |handle: ExtensionHandle| {
        // Add a context-menu item that copies the request URL
        handle.extend_context_menu_single(ContextMenuItemSingle::new(
            "copy-url",
            "Copy URL",
            async |ctx: SingleEntryContext, handle: ExtensionHandle| {
                let entry = handle
                    .get_session_entry(ctx.session_id, ctx.entry_id)
                    .await?;

                if let Some(entry) = entry {
                    handle.write_text_to_clipboard(&entry.url.to_string()).await?;
                }
                Ok(())
            },
        )).await?;

        // Add an overview field that shows the response content length
        handle.extend_overview(OverviewField::new(
            "content-length",
            "Content Length",
            async |ctx: SingleEntryContext, handle: ExtensionHandle| {
                let entry = handle
                    .get_session_entry(ctx.session_id, ctx.entry_id)
                    .await?;

                let size = entry
                    .and_then(|entry| entry.response)
                    .and_then(|res| res.body_size)
                    .map(|size| size.to_string());
                Ok(size)
            },
        )).await?;

        Ok(())
    })
    .await
}
```

