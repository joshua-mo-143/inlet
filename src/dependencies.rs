use crate::cli::Config;
use std::fs;
use std::path::PathBuf;
use toml_edit::{value, Array, Document};

const SHUTTLE_VERSION: &str = "0.24.0";

pub fn add_required_dependencies(project_path: PathBuf, cfg: Config) -> Result<(), &'static str> {
    let cargo_toml = fs::read_to_string(project_path.join("Cargo.toml"))
        .unwrap()
        .parse::<String>()
        .unwrap();

    let mut toml = match cargo_toml.parse::<Document>() {
        Ok(res) => res,
        Err(_e) => return Err("Meme!"),
    };

    toml.add_dependency("axum", "0.6.18");
    toml.add_dependency("shuttle-runtime", SHUTTLE_VERSION);
    toml.add_dependency("shuttle-axum", SHUTTLE_VERSION);
    toml.add_dependency("tokio", "1.28.2");

    if cfg.crud | cfg.auth {
        toml.add_dependency_with_features(
            "sqlx",
            "0.7.1",
            make_features(vec!["runtime-tokio-native-tls", "postgres", "chrono"]),
        );
    }

    if cfg.auth {
        toml.add_dependency_with_features(
            "axum-extra",
            "0.7.7",
            make_features(vec!["cookie-private"]),
        );
        toml.add_dependency_with_features(
            "chrono",
            "0.4.26",
            make_features(vec!["clock", "serde"]),
        );
        toml.add_dependency_with_features("serde", "1.0.171", make_features(vec!["derive"]));
        toml.add_dependency("time", "0.3.26");
        toml.add_dependency("bcrypt", "0.15.0");
        toml.add_dependency_with_features(
            "shuttle-shared-db",
            SHUTTLE_VERSION,
            make_features(vec!["postgres"]),
        );
    }

    if cfg.secrets {
        toml.add_dependency("shuttle-secrets", SHUTTLE_VERSION);
    }

    fs::write(project_path.join("Cargo.toml"), toml.to_string()).unwrap();

    Ok(())
}

trait ManageDependencies {
    fn add_dependency(&mut self, _name: &str, _version: &str) {}
    fn add_dependency_with_features(&mut self, _name: &str, _version: &str, _features: Array) {}
}

impl ManageDependencies for Document {
    fn add_dependency(&mut self, name: &str, version: &str) {
        self["dependencies"][name] = value(version);
    }

    fn add_dependency_with_features(&mut self, name: &str, version: &str, features: Array) {
        self["dependencies"][name]["version"] = value(version);
        self["dependencies"][name]["features"] = value(features);
    }
}

fn make_features(features: Vec<&str>) -> Array {
    let mut arr = Array::new();

    for feature in features {
        arr.push(feature);
    }

    arr
}
