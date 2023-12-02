use std::net::{TcpListener, TcpStream};

use anyhow::{Context, Result};
use ash_common::Request;
use serde::Deserialize;
use serde_json::Deserializer;

const LOCALHOST: &str = "127.0.0.1:0";

fn main() -> Result<()> {
    init_logging()?;

    let listener = TcpListener::bind(LOCALHOST).context("couldn't bind to socket")?;

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                log::error!("connection failed: {err}");
                continue;
            }
        };

        std::thread::spawn(|| {
            if let Err(err) = handle_connection(stream) {
                log::error!("{err}");
            }
        });
    }

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
        .chain(std::io::stderr())
        .chain(fern::log_file("output-server.log")?)
        .apply()?;

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    #[allow(clippy::never_loop)]
    loop {
        let mut de = Deserializer::from_reader(&mut stream);
        let request = Request::deserialize(&mut de)?;

        match request {
            Request::Quit => return Ok(()),
        }
    }
}
