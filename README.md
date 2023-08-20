# Inlet: A procedural CLI web app bootstrapper.
Inlet aims to be a framework-agnostic, procedural CLI web app bootstrapper for Rust web frameworks that leverages the power of packages like `syn` and `quote` to bring you a fully complete experience for the terminal that lets you bootstrap web apps that are production-ready.

## How does Inlet work?
Inlet works by using the power of AST traversal to be able to generate your code. 

## Roadmap
[X] Full CRUD route support
[X] Database-backed session route support
[ ] Implement auth middleware route
[ ] Bring the Inlet experience to Actix-web
[ ] Implement Oauth 
[ ] Implement payment routes
[ ] Support for generating SQL files at bootstrap-time

## Dependencies
chrono: Date-time stuff (get timestamp)
clap: CLI
indoc: Multiline string formatting
inquire: Multi-select
prettyplease: Unparsing syn back to text so it can be reasonably read
proc-macro2: Core component of this crate
quote: Core component of this crate
syn: Core component of this crate
toml_edit: Parsing the Cargo.toml file to add dependencies