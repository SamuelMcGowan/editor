use crate::peer::{Peer, PeerResult};
use crate::protocol::{Request, Response};

pub fn run() -> PeerResult<()> {
    let addr = crate::session::get_session_addr()?;

    let mut client = Peer::new(addr)?;

    client.send(Request::Quit)?;
    let response = client.receive::<Response>()?;

    log::info!("response: {response:?}");

    Ok(())
}
