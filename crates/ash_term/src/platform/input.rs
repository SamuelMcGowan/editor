use std::io::Read;
use std::time::Instant;
use std::{fmt, io};

use crossbeam_channel::{Receiver, RecvTimeoutError};

#[derive(Default, Clone)]
pub(crate) struct Bytes {
    buf: [u8; 32],
    len: usize,
}

impl Bytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    // pub fn len(&self) -> usize {
    //     self.len
    // }

    // pub fn is_empty(&self) -> bool {
    //     self.len == 0
    // }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = String::from_utf8_lossy(self.as_slice());
        s.fmt(f)
    }
}

pub(crate) struct PollingStdin {
    recv: Receiver<io::Result<Bytes>>,
}

impl Default for PollingStdin {
    fn default() -> Self {
        Self::new()
    }
}

impl PollingStdin {
    pub fn new() -> Self {
        let (send, recv) = crossbeam_channel::bounded(8);

        let mut stdin = io::stdin();
        std::thread::spawn(move || loop {
            let mut bytes = Bytes::default();

            match stdin.read(&mut bytes.buf) {
                // Some bytes were written, so send them to the main thread.
                Ok(len) => {
                    bytes.len = len;
                    send.send(Ok(bytes)).unwrap();
                }

                // Interrupted - continue reading.
                Err(err) if err.kind() == io::ErrorKind::Interrupted => {}

                // Eww, an error.
                Err(err) => send.send(Err(err)).unwrap(),
            }
        });

        Self { recv }
    }

    pub fn read_with_deadline(&self, deadline: Instant) -> io::Result<Option<Bytes>> {
        match self.recv.recv_deadline(deadline) {
            Ok(bytes) => bytes.map(Some),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(io::Error::new(
                io::ErrorKind::Other,
                "input thread disconnected",
            )),
        }
    }
}
