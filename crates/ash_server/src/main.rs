use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::{Context, Result};
use ash_server::{Request, Response};
use serde_json::Deserializer;

pub const LOCALHOST: &str = "127.0.0.1:0";

// --lock

fn main() -> Result<()> {
    init_logging()?;

    let project_dirs = ash_server::project_dirs().context("couldn't get project directories")?;

    std::fs::create_dir_all(project_dirs.data_dir())
        .context("couldn't create session data directory")?;

    let session_file_path = project_dirs.data_dir().join("session");

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&session_file_path)
    {
        Ok(file) => {
            let res = run_server(file);
            std::fs::remove_file(session_file_path).context("couldn't delete session file")?;
            res
        }
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            log::info!("server already running (session file exists)");
            Ok(())
        }
        Err(err) => Err(anyhow::Error::from(err).context("couldn't create session file")),
    }
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
        .chain(fern::log_file("logs/server.log")?)
        .apply()?;

    Ok(())
}

fn run_server(mut session_file: File) -> Result<()> {
    let listener = TcpListener::bind(LOCALHOST).context("couldn't bind to port")?;
    let addr = listener
        .local_addr()
        .context("couldn't get socket address")?;

    log::info!("listening on port {addr}");

    write!(session_file, "{}", addr.port()).context("couldn't write port to session file")?;

    for stream in listener.incoming() {
        let stream = stream.context("connection failed")?;
        std::thread::spawn(|| {
            if let Err(err) = handle_connection(stream) {
                log::error!("{}", err.context("while handling connection"));
            }
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    log::info!("connected to client: {}", stream.local_addr()?);

    let stream_read = stream.try_clone()?;

    for request in Deserializer::from_reader(stream_read).into_iter::<Request>() {
        let request = request?;

        log::info!("received request: {request:?}");

        let response = match request {
            Request::Quit => Response::Ok,
        };

        let response_json = serde_json::to_string(&response)?;

        write!(stream, "{response_json}")?;
        stream.flush()?;
    }

    log::info!("client {} disconnected", stream.peer_addr()?);

    Ok(())
}
