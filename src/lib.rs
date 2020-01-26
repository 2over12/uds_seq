extern crate nix;

pub use nix::sys::socket::SockType;
use nix::sys::socket::{self, AddressFamily, MsgFlags, SockAddr, SockFlag};

use std::io::{Read, Write};
use std::os::unix::io::RawFd;
use std::sync::Arc;
pub type Result<T> = std::result::Result<T, StreamError>;

#[derive(Debug)]
pub enum StreamError {
	PathDNE,
	InternalSys,
	BadError,
}

struct StreamFd(RawFd);

impl std::convert::From<nix::Error> for StreamError {
	fn from(err: nix::Error) -> StreamError {
		match err {
			nix::Error::InvalidPath => StreamError::PathDNE,
			nix::Error::Sys(_) => StreamError::InternalSys,
			_ => StreamError::BadError,
		}
	}
}

pub struct UnixStream {
	fd: Arc<StreamFd>,
}

pub struct UnixStreamRd {
	fd: Arc<StreamFd>,
}

pub struct UnixStreamWrt {
	fd: Arc<StreamFd>,
}

impl UnixStream {
	pub fn split(self) -> Result<(UnixStreamWrt, UnixStreamRd)> {
		Ok((
			UnixStreamWrt {
				fd: Arc::clone(&self.fd),
			},
			UnixStreamRd { fd: self.fd },
		))
	}

	pub fn new_with_type(path: &str, tp: SockType) -> Result<UnixStream> {
		let soc = socket::socket(AddressFamily::Unix, tp, SockFlag::empty(), None)?;
		let p = SockAddr::new_unix(path)?;
		socket::connect(soc, &p)?;

		Ok(UnixStream {
			fd: Arc::new(StreamFd(soc)),
		})
	}
	pub fn new(path: &str) -> Result<UnixStream> {
		UnixStream::new_with_type(path, SockType::SeqPacket)
	}
}

impl Drop for StreamFd {
	fn drop(&mut self) {
		socket::shutdown(self.0, socket::Shutdown::Both).unwrap_or(());
	}
}

impl Read for UnixStream {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		match socket::recv(self.fd.0, buf, MsgFlags::empty()) {
			Ok(sz) => Ok(sz),
			_ => Err(std::io::Error::last_os_error()),
		}
	}
}

impl Read for UnixStreamRd {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		match socket::recv(self.fd.0, buf, MsgFlags::empty()) {
			Ok(sz) => Ok(sz),
			_ => Err(std::io::Error::last_os_error()),
		}
	}
}

impl Write for UnixStreamWrt {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		match socket::send(self.fd.0, buf, MsgFlags::empty()) {
			Ok(sz) => Ok(sz),
			_ => Err(std::io::Error::last_os_error()),
		}
	}

	// Is this right?
	fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
		Ok(())
	}
}

impl Write for UnixStream {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		match socket::send(self.fd.0, buf, MsgFlags::empty()) {
			Ok(sz) => Ok(sz),
			_ => Err(std::io::Error::last_os_error()),
		}
	}

	// Is this right?
	fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
		Ok(())
	}
}

#[cfg(test)]
mod tests {

	#[test]
	fn it_works() {
		assert_eq!(1 + 1, 2);
	}
}
