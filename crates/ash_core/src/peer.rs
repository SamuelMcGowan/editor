use std::io;
use std::marker::PhantomData;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Deserializer, Serializer, StreamDeserializer};

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

pub struct Peer<Send, Recv> {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,

    write: Serializer<TcpStream>,
    read: StreamDeserializer<'static, serde_json::de::IoRead<TcpStream>, Recv>,

    _phantom: PhantomData<*const (Send, Recv)>,
}

impl<Send: Serialize, Recv: DeserializeOwned> Peer<Send, Recv> {
    pub fn from_addrs(addr: impl ToSocketAddrs) -> PeerResult<Self> {
        Self::from_stream(TcpStream::connect(addr)?)
    }

    pub fn from_stream(stream: TcpStream) -> PeerResult<Self> {
        let stream2 = stream.try_clone()?;

        let local_addr = stream.local_addr()?;
        let peer_addr = stream.peer_addr()?;

        Ok(Self {
            local_addr,
            peer_addr,

            write: Serializer::new(stream),
            read: Deserializer::from_reader(stream2).into_iter(),

            _phantom: PhantomData,
        })
    }

    pub fn send(&mut self, value: Send) -> PeerResult<()> {
        value.serialize(&mut self.write)?;
        Ok(())
    }

    pub fn receive(&mut self) -> PeerResult<Option<Recv>> {
        self.read.next().transpose().map_err(PeerError::from)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }
}
