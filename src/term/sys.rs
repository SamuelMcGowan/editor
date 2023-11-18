use std::fs::File;
use std::io::Write;
use std::mem::ManuallyDrop;
use std::os::fd::{FromRawFd, RawFd};
use std::{io, mem};

use libc::{termios as Termios, winsize as Winsize, STDIN_FILENO, STDOUT_FILENO};

macro_rules! cvt {
    ($res:expr) => {{
        match $res {
            -1 => Err(io::Error::last_os_error()),
            res => Ok(res),
        }
    }};
}

unsafe fn get_termios(fd: RawFd) -> io::Result<Termios> {
    unsafe {
        let mut termios: Termios = mem::zeroed();
        cvt!(libc::tcgetattr(fd, &mut termios))?;
        Ok(termios)
    }
}

unsafe fn set_termios(fd: RawFd, termios: &Termios) -> io::Result<()> {
    cvt!(unsafe { libc::tcsetattr(fd, libc::TCSANOW, termios) })?;
    Ok(())
}

unsafe fn get_size(fd: RawFd) -> io::Result<(u16, u16)> {
    let mut size: Winsize = unsafe { mem::zeroed() };
    cvt!(unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut size) })?;
    Ok((size.ws_col, size.ws_row))
}

pub struct RawTerm {
    termios_prev: Termios,
}

impl RawTerm {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let mut termios = get_termios(STDIN_FILENO)?;
            let termios_prev = termios;

            libc::cfmakeraw(&mut termios);

            Ok(Self { termios_prev })
        }
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        unsafe { get_size(STDIN_FILENO) }
    }
}

impl Drop for RawTerm {
    fn drop(&mut self) {
        let _ = unsafe { set_termios(STDIN_FILENO, &self.termios_prev) };
    }
}

pub struct RawStdout;

impl Write for RawStdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        get_stdout().write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice]) -> io::Result<usize> {
        get_stdout().write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        get_stdout().flush()
    }
}

fn get_stdout() -> ManuallyDrop<File> {
    ManuallyDrop::new(unsafe { File::from_raw_fd(STDOUT_FILENO) })
}
