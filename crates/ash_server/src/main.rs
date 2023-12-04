use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};

use anyhow::{Context, Result};
use ash_core::peer::{Peer, PeerResult};
use ash_core::protocol::{Request, Response};
use ash_core::session::SessionLock;

fn main() -> Result<()> {
    init_logging()?;

    let mut session = SessionLock::new()?;

    let listener = TcpListener::bind(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0))
        .context("couldn't bind to port")?;

    let addr = listener
        .local_addr()
        .context("couldn't get socket address")?;

    session.set_addr(addr)?;

    log::info!("listening on port {addr}");

    for stream in listener.incoming() {
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
        .chain(fern::log_file("logs/server.log")?)
        .apply()?;

    Ok(())
}

fn handle_connection(stream: std::io::Result<TcpStream>) -> PeerResult<()> {
    let mut peer = Peer::from_stream(stream?)?;

    log::info!("connected to client: {}", peer.local_addr());

    while let Some(request) = peer.receive()? {
        let response = match request {
            Request::Quit => Response::Ok,
        };

        peer.send(response)?;
    }

    log::info!("client {} disconnected", peer.peer_addr());

    Ok(())
}
