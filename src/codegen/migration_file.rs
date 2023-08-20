use indoc::{formatdoc, indoc};
use crate::cli::Config;
use std::fs;
use chrono::Utc;
use std::path::PathBuf;

pub fn write_migration_file(migrations_dir: PathBuf, cfg: Config) {

    let mut migrations_up = String::new();
    let mut migrations_down = String::new();
    
    if let Some(routes) = cfg.routes {
        for route in routes {
        migrations_up.push_str(&formatdoc! {"CREATE TABLE IF NOT EXISTS {route} (
                    id SERIAL PRIMARY KEY,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP    
                );\n\n"
        });

        migrations_down.push_str(&formatdoc! {"DROP TABLE {route};\n"});
        }

    }
    
        let timestamp = Utc::now().naive_local().format("%Y%m%d%H%M%S");

        let filename_up = format!("{timestamp}_schema.up.sql");
        let filename_down = format!("{timestamp}_schema.down.sql");

        if cfg.auth {
        migrations_up.push_str(indoc! {"CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            password VARCHAR NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS usersessions (
            id SERIAL PRIMARY KEY,
            user_id INT NOT NULL UNIQUE,
            session_id VARCHAR NOT NULL UNIQUE,
            expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
            );\n\n"
        });

        migrations_down.push_str(indoc! {"DROP TABLE users;
            DROP TABLE sessions;\n"
        })
    }

    fs::write(migrations_dir.join(filename_up), migrations_up).unwrap();
    fs::write(migrations_dir.join(filename_down), migrations_down).unwrap();
}