// The goal of this example is to show an implementation of the library on Unix-based OSes
// In order to use the library, one must have the TcpClient trait implemented.
// Implementing the Trait using TcpStream (std::net) hasn't been really straightforward due
// to the constraints brought by the Trait asking to create an unused socket, which is not possible
// with TcpStream (check out `socket` function).

use embedded_nal::{TcpClient, SocketAddr, nb};
use std::net::{TcpStream, Shutdown};
use std::str::FromStr;
use std::io::{Write, Read};
use std::time::Duration;
use leger::{Provider, ProviderError, TcpError};

pub struct UnixTcpStack {
}

impl TcpClient for UnixTcpStack {
	type TcpSocket = TcpStream;
	type Error = TcpError;

	// We want the socket to be created but we don't want any connection
	// using TcpStream don't allow to do so, so I am calling the connect function
	// to a default address
	// TODO, there should be a better way to handle this!
	fn socket(&self) -> Result<Self::TcpSocket, Self::Error> {
		let addrs = [ std::net::SocketAddr::from(([127, 0, 0, 1], 9944)) ];

		let socket = TcpStream::connect(&addrs[..]).unwrap();
		Ok(socket)
	}

	fn connect(&self, socket: &mut Self::TcpSocket, remote: SocketAddr) -> nb::Result<(), Self::Error> {
		let addrs = [ std::net::SocketAddr::from_str(remote.to_string().as_str()).unwrap() ];

		let mut socket_cpy = TcpStream::connect(&addrs[..]).unwrap();
		std::mem::swap(socket, &mut socket_cpy);

		socket.set_read_timeout(Some(Duration::new(2, 0))).expect("set_read_timeout call failed");
		Ok(())
	}

	fn is_connected(&self, _socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
		// It's not possible to use a disconnected TcpStream as we need to connect to create it
		Ok(true)
	}

	fn send(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
		if let Ok(written) = socket.write(buffer) {
			Ok(written)
		} else {
			Err(nb::Error::WouldBlock)
		}
	}

	fn receive(&self, socket: &mut Self::TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
		if let Ok(read) = socket.read(buffer) {
			Ok(read)
		} else {
			Err(nb::Error::WouldBlock)
		}
	}

	fn close(&self, socket: &Self::TcpSocket) -> Result<(), Self::Error> {
		// It is advised to drop the socket reference to make sure it is closed
		// [The connection will be closed when the value is dropped](https://doc.rust-lang.org/beta/std/net/struct.TcpStream.html)
		if let Ok(_) = socket.shutdown(Shutdown::Both) {
			Ok(())
		} else {
			Err(TcpError::CannotClose)
		}
	}
}


fn main() -> Result<(), ProviderError> {
	let tcp = UnixTcpStack{	};
	let mut pp: Provider<TcpStream> = Provider::new(&tcp, "127.0.0.1:9944")?;

	let name = pp.chain_info()?;
	println!("🧪 Name: {}", name);

	let genesis = pp.system_version()?;
	println!("✌️ Version {}", genesis);

	let resp = pp.genesis_hash()?;
	println!("🐥 Genesis block hash {}", resp);

	println!("🐥 Runtime version {}", pp.runtime_version()?);

	Ok(())
}