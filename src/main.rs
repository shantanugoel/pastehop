mod app;
mod cli;
mod clipboard;
mod config;
mod doctor;
mod errors;
mod gc;
mod hook;
mod install;
mod naming;
mod profiles;
mod staging;
mod target;
mod terminal;
mod transport;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(error) = app::run(cli) {
        eprintln!("{error:#}");
        std::process::exit(exit_code_from_error(&error));
    }
}

fn exit_code_from_error(error: &anyhow::Error) -> i32 {
    error
        .chain()
        .find_map(|cause| {
            cause
                .to_string()
                .strip_prefix("exit_code=")
                .and_then(|value| value.parse::<i32>().ok())
        })
        .unwrap_or(1)
}
