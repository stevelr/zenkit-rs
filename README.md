Zenkit client library in Rust with caching. Supports
most types and functions of the documented Zenkit REST API, plus helper functions
that make it easier to read, update, and create list items. This library
has been used to make client and server apps, WASM libraries, and webhooks.

Documentation is at [docs.rs](https://docs.rs/zenkit).
Official reference at [Zenkit API docs](https://base.zenkit.com/docs/api/overview/introduction)

## Rust source-code generation

If you are writing a Rust client for Zenkit, check out
[zk-codegen](https://github.com/stevelr/zenkit-codegen),
which can generate a client library (wrapping this library),
with an API derived from the lists and fields defined in your workspace.

## Companion cli tool

[zenkit-cli](https://github.com/stevelr/zenkit-cli) has command-line
capabilities that may be useful for testing and automation.
It may also be useful to view as example code for many of the functions
in this library.

