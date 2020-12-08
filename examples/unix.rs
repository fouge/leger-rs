// The goal of this example is to show an implementation of the library on Unix-based OSes
// In order to use the library, one must have the TcpClient trait implemented.
// Implementing the Trait using TcpStream (std::net) hasn't been really straightforward due
// to the constraints brought by the Trait asking to create an unused socket, which is not possible
// with TcpStream (check out `socket` function).

use embedded_nal::{TcpClientStack, SocketAddr, nb};
use std::net::{TcpStream, Shutdown};
use std::str::{FromStr};
use std::io::{Write, Read};
use std::time::Duration;
use leger::{Provider, ProviderError, TcpError};
use leger::chain::Chain;
use leger::account::{Account};
use leger::calls::Calls;

pub struct UnixTcpStack {
}

impl TcpClientStack for UnixTcpStack {
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
	let mut seed:[u8; 32] = [0_u8; 32];
	// Use Alice account
	// 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
	// secret: e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
	hex::decode_to_slice(
		"e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		&mut seed as &mut [u8])
		.expect("Cannot decode hex string");
	let tcp = UnixTcpStack{	};
	let mut pp: Provider<TcpStream> = Provider::new(&tcp, "127.0.0.1:9944")?;

	let mut account = Account::new(seed);

	let name = pp.system_name()?;
	println!("🧪 Name: {}", name);

	let genesis = pp.system_version()?;
	println!("✌️ Version {}", genesis);

	println!("🐥 Runtime version {}", pp.runtime_version()?);

	let resp = pp.get_genesis_block_hash()?;
	println!("🐥 Genesis block hash 0x{:02x?}", resp);

	let resp = pp.get_block_hash(None)?;
	println!("🏷 Last block hash 0x{:02x?}",resp);

	let resp = pp.get_finalized_head()?;
	println!("🤖 Finalized head {}", resp);

	println!("🔑 Using account: {}", account.ss58());

	let resp = account.get_info(&mut pp);
	if let Ok(r) = resp {
		println!("💰 {:?}", r);
	}

	let resp = account.get_balance(&mut pp);
	match resp {
		Ok(ba) => {
			println!("💰 Balance: {}", ba);
		}
		Err(e) => {
			eprintln!("Error {:?}", e);
		}
	}

	// Sending to bob
	// ss58: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
	let mut dest_account:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48",
		&mut dest_account as &mut [u8])
		.expect("Cannot decode hex string");

	let amount_to_send = 2921503981796281;
	println!("🤑 Sending {} units to Bob: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", amount_to_send);

	let resp = pp.balance_transfer(&mut account, &dest_account, amount_to_send);
	println!("🔗 Finalized block hash: {:?}", resp.unwrap());

	Ok(())
}