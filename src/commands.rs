use prettyplease::unparse;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use syn::File;

pub fn cargo_init(project_name: &str) -> PathBuf {
    Command::new("cargo")
        .args(["init", "--bin", project_name])
        .output()
        .expect("Failed to init cargo");

    fs::canonicalize(project_name).unwrap()
}

pub fn make_dir(project_path: PathBuf, foldername: &str) -> PathBuf {
    let folderdir = project_path.join(format!("src/{foldername}"));

    fs::create_dir(folderdir.clone()).unwrap();

    fs::canonicalize(folderdir).unwrap()
}

pub fn write_file(code: File, file_loc: PathBuf) -> Result<(), String> {
    let fmt = unparse(&code);

    std::fs::write(file_loc, fmt).unwrap();

    Ok(())
}

pub fn write_secrets_file(project_dir: PathBuf) {
    let fmt = r#"KEY = "VALUE""#;

    fs::write(project_dir.join("Secrets.toml"), fmt).unwrap();
}

pub fn write_main_file(
    code: File,
    mut dynamic_deps: String,
    file_loc: PathBuf,
) -> Result<(), String> {
    let fmt = unparse(&code);

    dynamic_deps.push_str(&fmt);

    std::fs::write(file_loc, dynamic_deps).unwrap();

    Ok(())
}

pub fn write_mod_file(dir_loc: PathBuf) -> Result<(), String> {
    let mut modules = Vec::new();
    let paths = fs::read_dir(dir_loc.clone()).unwrap();

    for path in paths {
        let filename = path
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .replace(".rs", "");
        modules.push(Ident::new(&filename, Span::call_site()));
    }

    let mod_file_contents = quote! {
        #(
            pub mod #modules;
        )*
    };

    let mod_file_contents = syn::parse_file(&mod_file_contents.to_string()).unwrap();

    write_file(mod_file_contents, dir_loc.join("mod.rs")).unwrap();

    Ok(())
}
