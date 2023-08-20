use cli::process_commands;

mod cli;
mod codegen;
mod commands;
mod dependencies;

fn main() {
    if let Err(e) = process_commands() {
        panic!("Process panicked: {e}");
    }

    println!("Bootstrapping complete!");
}
