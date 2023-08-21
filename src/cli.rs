use crate::codegen::axum_auth::{auth_middleware, auth_routes};
use crate::codegen::main_fn::{axum_crud_fns, main_function};
use crate::codegen::migration_file::write_migration_file;
use crate::commands::{
    cargo_init, make_dir, write_file, write_main_file, write_mod_file, write_secrets_file,
};
use crate::dependencies::add_required_dependencies;
use inquire::{Confirm, Text};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmds: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Create {
        /// Creates API routes for each name inputted (comma separated)
        #[arg(short, long)]
        crud: Option<Vec<String>>,
        /// Creates auth routes using database-backed cookie session auth
        #[arg(short, long)]
        auth: bool,
        /// Adds a secrets file that you can use to hold secrets as well as the shuttle-secrets crate.
        #[arg(short, long)]
        secrets: bool,
        /// Adds the name of your project.
        #[arg(short, long)]
        name: Option<String>,
    },
    Test,
}

pub fn process_commands() -> Result<(), String> {
    let cli = Cli::parse();

    let mut cfg = Config {
        crud: false,
        auth: false,
        routes: None,
        secrets: false,
    };

    match cli.cmds {
        Some(Commands::Create {
            crud,
            auth,
            secrets,
            name,
        }) => {
            let project_name = match name {
                Some(res) => res,
                None => {
                    let name = Text::new("Hey there! What would you like to name your project? > ")
                        .prompt();

                    match name {
                        Ok(name) => name,
                        Err(_) => return Err("Couldn't find a name :(".to_string()),
                    }
                }
            };

            let protected = match auth && crud.is_some() {
               true => { 
                let prompt = Confirm::new("Do you want to protect your CRUD routes?").prompt();

                match prompt {
                    Ok(true) => true,
                    Ok(false) => false,
                    Err(_) => return Err("Couldn't find your choice :(".to_string())
                }
                    },
                false => false
            };

            let project_path = cargo_init(&project_name);
            let routes_dir = make_dir(project_path.clone(), "routes");

            let mut routes: Vec<Route> = Vec::new();

            cfg.secrets = secrets;

            if auth {
                cfg.auth = auth;
                let middleware_dir = make_dir(project_path.clone(), "middleware");
                write_file(auth_middleware(), middleware_dir.join("auth.rs")).unwrap();
                write_mod_file(middleware_dir).unwrap();
                write_file(auth_routes(), routes_dir.clone().join("auth.rs")).unwrap();
            }

            if let Some(res) = crud {
                cfg.crud = true;
                for name in res {
                    routes.push(Route {
                        name,
                        auth_required: protected,
                    });
                }

                for route in &routes {
                    let tablename_as_filename = format!("{}.rs", route.name);

                    let (crud_routes, extra_deps) = if route.auth_required {
                        axum_crud_fns(route.clone(), true)
                    } else {
                        axum_crud_fns(route.clone(), false)
                    };

                    write_main_file(
                        crud_routes,
                        extra_deps,
                        routes_dir.clone().join(tablename_as_filename),
                    )
                    .unwrap();
                }
            }

            if cfg.crud | cfg.auth {
                let migrations_dir = make_dir(project_path.clone(), "migrations");
                write_migration_file(migrations_dir, cfg.clone());
            }

            if cfg.secrets {
                write_secrets_file(project_path.clone());
            }

            write_mod_file(routes_dir).unwrap();

            let (main_fn_file, router_useitems) = main_function(cfg.clone(), routes);
            write_main_file(
                main_fn_file,
                router_useitems,
                project_path.join("src/main.rs"),
            )
            .unwrap();

            if let Err(e) = add_required_dependencies(project_path, cfg) {
                panic!("Error while adding dependencies: {e}");
            }
        }
        Some(Commands::Test) => {}
        None => {}
    }

    Ok(())
}

#[derive(Clone)]
pub struct Config {
    pub crud: bool,
    pub auth: bool,
    pub routes: Option<Vec<String>>,
    pub secrets: bool,
}

#[derive(Clone)]
pub struct Route {
    pub name: String,
    pub auth_required: bool,
}
