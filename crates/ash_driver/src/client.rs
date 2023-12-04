use std::io;
use std::net::{TcpStream, ToSocketAddrs};

use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};

use crate::project_dirs;
use crate::protocol::{Request, Response};

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // TODO: remove
    #[error("session missing")]
    SessionMissing,
    #[error("session invalid")]
    SessionInvalid,
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

pub fn run() -> ClientResult<()> {
    let project_dirs = project_dirs().ok_or(ClientError::SessionMissing)?;
    let session_file_path = project_dirs.data_dir().join("session");

    let port = std::fs::read_to_string(session_file_path)?;
    let port = port
        .parse::<u16>()
        .map_err(|_| ClientError::SessionInvalid)?;

    let mut client = Client::new(("localhost", port))?;

    let response = client.send(Request::Quit)?;
    log::info!("response: {response:?}");

    Ok(())
}
