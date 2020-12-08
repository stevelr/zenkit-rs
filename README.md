Rust client library for reading and updating Zenkit workspaces

Documentation is at [docs.rs](https://docs.rs/zenkit).
Also see the official reference at [Zenkit API docs](https://base.zenkit.com/docs/api/overview/introduction)

This library has been used for native apps and web apps,
and has been used to implement Zenkit Webhooks 
in WASM on Cloudflare Workers using [wasm-service].

## Code generation

If you are writing a Rust client for Zenkit, check out
[zk-codegen](https://github.com/stevelr/zenkit-codegen),
which can generate a client library (wrapping this library),
with an API derived from the lists and fields defined in your workspace.

## Companion cli tool

[zenkit-cli](https://github.com/stevelr/zenkit-cli) has command-line
capabilities that may be useful for testing and automation.
It also can serve as example code for most of the functions
in this library.

