pub mod client;
pub mod protocol;

use clap::{Parser, Subcommand};
use directories::ProjectDirs;

fn main() -> Result<(), DriverError> {
    init_logging()?;

    let args = Cli::parse();

    match args.command.unwrap_or_default() {
        CliCommand::Client => {
            log::info!("starting client");
            client::run()?;
        }

        CliCommand::Server => {
            log::info!("starting server");
        }
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum DriverError {
    #[error(transparent)]
    Logger(#[from] fern::InitError),

    #[error(transparent)]
    Client(#[from] client::ClientError),
}

fn init_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let now = chrono::Local::now();

            out.finish(format_args!(
                "[{} {} {}] {}",
                now.format("%Y/%m/%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stderr()) // remove before doing terminal stuff
        .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand, Default)]
enum CliCommand {
    #[default]
    Client,
    Server,
}

// TODO: remove
pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "ash_editor")
}
