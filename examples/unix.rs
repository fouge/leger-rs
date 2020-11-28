// The goal of this example is to show an implementation of the library on Unix-based OSes
// In order to use the library, one must have the TcpClient trait implemented.
// Implementing the Trait using TcpStream (std::net) hasn't been really straightforward due
// to the constraints brought by the Trait asking to create an unused socket, which is not possible
// with TcpStream (check out `socket` function).

use embedded_nal::{TcpClient, SocketAddr, nb};
use std::net::TcpStream;
use embedded_dot::{PolkaProviderError, PolkaProvider};
use std::mem::MaybeUninit;
use std::ops::Deref;
use no_std_net::ToSocketAddrs;


pub struct UnixTcpStack {
}

impl core::fmt::Debug for UnixTcpStack {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "Hi")
	}
}

impl ToSocketAddrs for SocketAddr {
	type Iter = ();

	fn to_socket_addrs(&self) -> Result<Self::Iter, ToSocketAddrError> {
		unimplemented!()
	}
}

impl TcpClient for UnixTcpStack {
	type TcpSocket = TcpStream;
	type Error = PolkaProviderError;

	// We want the socket to be created but we don't want any connection
	// using TcpStream don't allow to do so, so I am calling the connect function
	// to a default address
	// TODO, there should be a better way to handle this!
	fn socket(&self) -> Result<Self::TcpSocket, Self::Error> {
		let addrs = [ std::net::SocketAddr::from(([127, 0, 0, 1], 9944)) ];

		let mut socket = TcpStream::connect(&addrs[..]).unwrap();
		Ok(socket)
	}

	fn connect(&self, socket: &mut Self::TcpSocket, remote: SocketAddr) -> nb::Result<(), Self::Error> {
		let mut socket = TcpStream::connect(remote).unwrap();
		Ok(())
	}

	fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
		unimplemented!()
	}

	fn send(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
		unimplemented!()
	}

	fn receive(&self, socket: &mut Self::TcpSocket, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
		unimplemented!()
	}

	fn close(&self, socket: &Self::TcpSocket) -> Result<(), Self::Error> {
		unimplemented!()
	}
}


fn main() -> Result<(), PolkaProviderError> {
	let mut tcp = UnixTcpStack{	};
	let mut pp:PolkaProvider<TcpStream> = PolkaProvider::new(&tcp);

	pp.connect("hgell");

	Ok(())
}