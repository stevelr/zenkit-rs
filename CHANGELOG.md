# Changelog for zenkit-rs

## 0.4 - unreleased

- Added/updated documentation
- Breaking api changes:
  - moved Item,ListInfo and field setters into types module for consistency.
  - don't re-export std::result::Result
  - changed Activity.element_name from String to Option<String>
- Added Error.is_rate_limit() to flag errors due to rate limits
- Added trait types::ZKObjectID, re-exported in prelude
- Removed item.take_text_value(), which requires mutable item but items
  in public api are immutable
- Removed ApiClient.get_http() and log_object(), used during development
- Removed explicit impl Send/Sync for List,Workspace, because they are automatically derived
- Removed duplicate code for ChangedData and Entry.get_*
