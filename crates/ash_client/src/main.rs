use std::io::{self};
use std::net::{TcpStream, ToSocketAddrs};

use anyhow::{Context, Result};
use ash_server::{Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};

fn main() -> Result<()> {
    init_logging()?;

    let project_dirs = ash_server::project_dirs().context("couldn't get project directories")?;
    let session_file_path = project_dirs.data_dir().join("session");

    let port = std::fs::read_to_string(session_file_path).context("couldn't read session file")?;
    let port = port.parse::<u16>()?;

    let mut client = Client::new(("localhost", port))?;

    let response = client.send(Request::Quit)?;
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

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type ClientResult<T> = Result<T, ClientError>;

pub struct Client {
    write: Serializer<TcpStream>,
    read: Deserializer<serde_json::de::IoRead<TcpStream>>,
}

impl Client {
    pub fn new(addr: impl ToSocketAddrs) -> ClientResult<Self> {
        let stream = TcpStream::connect(addr)?;
        let stream2 = stream.try_clone()?;

        Ok(Self {
            write: Serializer::new(stream),
            read: Deserializer::from_reader(stream2),
        })
    }

    pub fn send(&mut self, request: Request) -> ClientResult<Response> {
        request.serialize(&mut self.write)?;
        Ok(Response::deserialize(&mut self.read)?)
    }
}
