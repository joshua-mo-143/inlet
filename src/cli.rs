use crate::codegen::main_fn::{axum_crud_routes, main_function};
use crate::codegen::axum_auth::auth_routes;
use crate::codegen::migration_file::write_migration_file;
use crate::commands::{cargo_init, make_dir, write_file, write_mod_file, write_main_file, write_secrets_file};
use crate::dependencies::add_required_dependencies;
use inquire::Text;

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

        name: Option<String>


    },
    Test
}

pub fn process_commands() -> Result<(), String> {
    let cli = Cli::parse();

    let mut cfg = Config {
        crud: false,
        auth: false,
        routes: None,
        secrets: false
    };

    match cli.cmds {
        Some(Commands::Create { crud, auth, secrets, name }) => {
            
                let project_name = match name {
                Some(res) => res,
                None => {
                    let name = Text::new("Hey there! What would you like to name your project? > ")
                        .prompt();

                    match name {
                        Ok(name) => name,
                        Err(_) => return Err("Couldn't detect a name :(".to_string())
                    }
                }
            };
    
            let project_path = cargo_init(&project_name);
            let routes_dir = make_dir(project_path.clone(), "routes");

            cfg.secrets = secrets;
            if let Some(res) = crud {
                cfg.crud = true;
                cfg.routes = Some(res.clone());
                
                for value in res {
                    let tablename_as_filename = format!("{value}.rs");

                    write_file(
                        axum_crud_routes(&value),
                        routes_dir.clone().join(tablename_as_filename),
                    )
                    .unwrap();
                }
            }
            if auth {
                cfg.auth = auth;
                write_file(auth_routes(), routes_dir.clone().join("auth.rs")).unwrap();
            }
            
            if cfg.crud | cfg.auth {
            let migrations_dir = make_dir(project_path.clone(), "migrations");
            write_migration_file(migrations_dir, cfg.clone());
            }

            if cfg.secrets {
                write_secrets_file(project_path.clone());
            }
            
            write_mod_file(routes_dir).unwrap();

            

            let (main_fn_file, router_useitems) = main_function(cfg.clone());
            write_main_file(main_fn_file, router_useitems, project_path.join("src/main.rs")).unwrap();
            
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
    pub secrets: bool
}
