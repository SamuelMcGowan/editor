use std::fs::OpenOptions;
use std::io::{self, ErrorKind, Write};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid session file contents")]
    ParseError,

    #[error("data directory missing")]
    DataDirMissing,

    #[error("session already exists")]
    SessionAlreadyExists,

    #[error("no active session")]
    SessionMissing,
}

pub type SessionResult<T> = Result<T, SessionError>;

pub struct SessionLock {
    session_file_path: PathBuf,
}

impl SessionLock {
    pub fn new(addr: SocketAddr) -> SessionResult<Self> {
        let data_dir = get_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;

        let session_file_path = data_dir.join("session");

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&session_file_path)
        {
            Ok(mut file) => {
                write!(file, "{addr}")?;
                file.flush()?;

                Ok(Self { session_file_path })
            }

            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                Err(SessionError::SessionAlreadyExists)
            }

            Err(err) => Err(SessionError::Io(err)),
        }
    }
}

impl Drop for SessionLock {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_file(&self.session_file_path) {
            log::error!("{err}");
        };
    }
}

fn get_data_dir() -> SessionResult<PathBuf> {
    let data_dir = dirs::data_dir().ok_or(SessionError::DataDirMissing)?;
    Ok(data_dir.join("ash_editor"))
}

pub fn get_session_addr() -> SessionResult<SocketAddr> {
    let data_dir = get_data_dir()?;
    let session_file_path = data_dir.join("session");

    let session_str = match std::fs::read_to_string(session_file_path) {
        Ok(s) => s,
        Err(err) if err.kind() == ErrorKind::NotFound => return Err(SessionError::SessionMissing),
        Err(err) => return Err(SessionError::Io(err)),
    };

    let addr = session_str
        .parse::<SocketAddr>()
        .map_err(|_| SessionError::ParseError)?;

    Ok(addr)
}
