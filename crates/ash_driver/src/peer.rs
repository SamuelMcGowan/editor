use std::io;
use std::net::{TcpStream, ToSocketAddrs};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Deserializer, Serializer};

use crate::session::SessionError;

#[derive(thiserror::Error, Debug)]
pub enum PeerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Session(#[from] SessionError),
}

pub type PeerResult<T> = Result<T, PeerError>;

pub struct Peer {
    write: Serializer<TcpStream>,
    read: Deserializer<serde_json::de::IoRead<TcpStream>>,
}

impl Peer {
    pub fn new(addr: impl ToSocketAddrs) -> PeerResult<Self> {
        let stream = TcpStream::connect(addr)?;
        let stream2 = stream.try_clone()?;

        Ok(Self {
            write: Serializer::new(stream),
            read: Deserializer::from_reader(stream2),
        })
    }

    pub fn send<T: Serialize>(&mut self, value: T) -> PeerResult<()> {
        value.serialize(&mut self.write)?;
        Ok(())
    }

    pub fn receive<T: DeserializeOwned>(&mut self) -> PeerResult<T> {
        Ok(T::deserialize(&mut self.read)?)
    }
}
