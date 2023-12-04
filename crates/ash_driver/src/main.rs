use clap::{Parser, Subcommand};

fn main() -> Result<(), DriverError> {
    init_logging()?;

    let args = Cli::parse();

    match args.command.unwrap_or_default() {
        CliCommand::Client => {
            log::info!("starting client");
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
        .chain(fern::log_file("logs/driver.log")?)
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
