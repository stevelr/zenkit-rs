# Changelog for zenkit-rs (https://github.com/stevelr/zenkit-rs)

## v0.5.0 2021-01-12

- upgraded to reqwest 0.11
  For non-wasm targets (e.g., cli tools), this makes tokio 1.0 a
  dependency.
- better error diagnostic when api token is unset
- Return types of some methods in types::Entry were simplified 
  from Result<Vec<String>,Error> to Vec<String>
- completed conversion of date fields from String/DateString to DateTime<Utc>

## v0.4.0 - 2020-12-18

- Added/updated documentation
- Breaking api changes:
  - moved Item,ListInfo and field setters into types module for consistency.
  - don't re-export std::result::Result
  - changed Activity.element_name from String to Option<String>
  - replaced DateString with DateTime<Utc> (re-exported from chrono)
- Added Error.is_rate_limit() to flag errors due to rate limits
- Added trait types::ZKObjectID, re-exported in prelude
- Added chrono as dependency
- Removed item.take_text_value(), which requires mutable item but items
  in public api are immutable
- Removed ApiClient.get_http() and log_object(), used during development
- Removed explicit impl Send/Sync for List,Workspace, because they are automatically derived
- Removed duplicate code for ChangedData and Entry.get_*
