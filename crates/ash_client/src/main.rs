use std::io::Write;
use std::net::TcpStream;

use anyhow::{Context, Result};
use ash_server::{Request, Response};
use serde::Deserialize;
use serde_json::Deserializer;

fn main() -> Result<()> {
    init_logging()?;

    let project_dirs = ash_server::project_dirs().context("couldn't get project directories")?;
    let session_file_path = project_dirs.data_dir().join("session");

    let port = std::fs::read_to_string(session_file_path).context("couldn't read session file")?;
    let port = port.parse::<u16>()?;

    let mut stream =
        TcpStream::connect(("127.0.0.1", port)).context("couldn't connect to server")?;

    log::info!("connected to server at {}", stream.peer_addr()?);

    let request = serde_json::to_string(&Request::Quit)?;
    log::info!("request: {request}");

    write!(stream, "{request}")?;
    stream.flush()?;

    let mut de = Deserializer::from_reader(stream);
    let response = Response::deserialize(&mut de).context("couldn't read response")?;

    log::info!("response: {response:?}");

    Ok(())
}

fn init_logging() -> Result<()> {
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
        .chain(fern::log_file("logs/client.log")?)
        .apply()?;

    Ok(())
}
