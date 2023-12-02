use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::{Context, Result};
use directories::ProjectDirs;

pub const LOCALHOST: &str = "127.0.0.1:0";

fn main() -> Result<()> {
    init_logging()?;

    let project_dirs =
        ProjectDirs::from("", "", "ash_editor").context("couldn't get project directories")?;

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
        Err(err) if err.kind() == ErrorKind::AlreadyExists => Ok(()),
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
        .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

struct Guard<F: FnMut()>(F);

impl<F: FnMut()> Drop for Guard<F> {
    fn drop(&mut self) {
        self.0()
    }
}

fn run_server(mut session_file: File) -> Result<()> {
    let listener = TcpListener::bind(LOCALHOST).context("couldn't bind to socket")?;
    let addr = listener
        .local_addr()
        .context("couldn't get socket address")?;

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

fn handle_connection(_stream: TcpStream) -> Result<()> {
    Ok(())
}
