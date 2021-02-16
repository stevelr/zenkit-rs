# Changelog for zenkit-rs (https://github.com/stevelr/zenkit-rs)

## v0.6.3 2021-02-16

- fix clippy warnings

## v0.6.2 2021-02-16

- updated dependencies: bytes (0.5 to 1.0), strum-macros (0.19-0.20)

## v0.6.1 2021-01-26

- fix: fixed parsing and deserialization of DateTime, so it now works 
  whether the field contains a full datetime or 
  just a date ("YYYY-MM-DD"). The shorter version can happen for fields
  that are set to allow date entry without time, in which case
  the parser/deserializer appends time 00:00:00

  Making this work required implementation of an internal DateTime,
  a tuple struct wrapping a chrono::DateTime. The crate already 
  exported DateTime, so for any uses of zenkit::DateTime, this 
  should not be a breaking change. zenkit::DateTime<Tz> also implements 
  Deref to chrono::DateTime<Tz>, so most uses of chrono::DateTime 
  methods on zenkit::DateTime should also work unchanged.

  Created unit tests for DateTime.

## v0.6.0 2021-01-23

- Activity.list_entry_description changed from String to Option<String>
  because it doesn't always appear in Activity events and breaks json
  deserialization. Because this is an api change, semver rules say 
  version must increase to 0.6.
- Added License files to repo. License (MIT OR Apache-2.0) is unchanged.

## v0.5.1 2020-01-17

- removed dependency on uuid crate

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
