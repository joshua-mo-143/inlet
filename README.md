# Inlet: A procedural Rust-based CLI web app bootstrapper.
Inlet aims to be a framework-agnostic, procedural CLI web app bootstrapper for Rust web frameworks that leverages the power of packages like `syn` and `quote` to bring you a fully complete experience for the terminal that lets you bootstrap web apps that are production-ready.

## How does Inlet work?
Inlet works by using the power of AST traversal to be able to generate your code. 

## Getting Started
To get started you'll want to have the following installed:
- The Rust language

This crate in its current state is mainly for use with [shuttle](https://www.shuttle.rs), so you'll want `cargo-shuttle` installed as well as Docker if you need a database.

Currently this crate isn't published to crates.io, but you can use this crate by git cloning the repo and using `cargo run -- create`.

Inlet currently supports the following flags:
```
--crud: Creates API routes for each name inputted (takes a string or string array)
--auth: Creates database-backed cookie session auth routes
--secrets: Adds a Secrets.toml file
--name: The name of your 
```
Once you execute the command, the sorcery will commence!

When Inlet is finished, you'll want to make sure to do the following:
- Make sure your migrations are what you want and add any structs you need for requests
- Install sqlx-cli and run migrations (or add the migrations macro to your main function!)
- Add any secrets you might need
- Add a frontend if you want

Once you're ready, you can deploy by running the following:
```sh
cargo shuttle project start && cargo shuttle deploy --allow-dirty
```

## Roadmap
- [x] Full CRUD route support
- [x] Database-backed session route support
- [ ] Implement auth middleware route
- [ ] Automagically add auth middleware to routes
- [ ] Bring the Inlet experience to Actix-web
- [ ] Implement Oauth 
- [ ] Implement payment routes
- [ ] Support for properly setting up SQL tables/migrations through initial prompt

## Dependencies
- chrono: Date-time stuff (get timestamp)
- clap: CLI
- indoc: Multiline string formatting
- inquire: Multi-select
- prettyplease: Unparsing syn back to text so it can be reasonably read
- proc-macro2: Core component of this crate
- quote: Core component of this crate
- syn: Core component of this crate
- toml_edit: Parsing the Cargo.toml file to add dependencies

## Issues/Contributions
Feel free to send in pull requests or issues for:
- Security/error handling issues
- Routing improvements
- Bugs
- Feature requests and suggestions
- Anything you think might benefit the repo

Do bear in mind this is primarily a one-man project at the moment. If you have an idea and it's not on the roadmap, it may be better to submit a PR yourself. 
However if you have an idea for something but you're having trouble implementing it yourself, feel free to reach out!

## License
MIT