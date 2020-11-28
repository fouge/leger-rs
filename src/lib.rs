#![cfg_attr(not(feature = "std"), no_std)]
#![no_builtins]

use embedded_websocket as ws;
use embedded_websocket::{WebSocketOptions, WebSocketSendMessageType, WebSocketReceiveMessageType, WebSocketCloseStatusCode};
use embedded_nal::{TcpClient, SocketAddrV4, Ipv4Addr, IpAddr};
use rand::rngs::ThreadRng;
use crate::PolkaProviderError::TcpSocket;


#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum TcpError {
	CountNotMatching,
	CannotClose,
	Unknown,
}

#[derive(Debug)]
pub enum PolkaProviderError {
	WebSocket(ws::Error),
	TcpSocket(TcpError),
	Embedded(embedded_nal::nb::Error<TcpError>),
	Utf8Error,
	Unknown
}

impl From<ws::Error> for PolkaProviderError {
	fn from(err: ws::Error) -> PolkaProviderError {
		PolkaProviderError::WebSocket(err)
	}
}

impl From<TcpError> for PolkaProviderError {
	fn from(err: TcpError) -> PolkaProviderError {
		PolkaProviderError::TcpSocket(err)
	}
}

impl From<embedded_nal::nb::Error<TcpError>> for PolkaProviderError {
	fn from(err: embedded_nal::nb::Error<TcpError>) -> PolkaProviderError {
		PolkaProviderError::Embedded(err)
	}
}

impl From<core::str::Utf8Error> for PolkaProviderError {
	fn from(err: core::str::Utf8Error) -> PolkaProviderError {
		PolkaProviderError::Utf8Error
	}
}

impl From<embedded_nal::nb::Error<PolkaProviderError>> for PolkaProviderError {
	fn from(err: embedded_nal::nb::Error<PolkaProviderError>) -> PolkaProviderError {
		if let embedded_nal::nb::Error::Other(e) = err {
			e
		} else {
			PolkaProviderError::Unknown
		}
	}
}

pub struct PolkaProvider<'a, S> {
	socket: S,
	ws: ws::WebSocketClient<ThreadRng>,
	in_buf: [u8; 4000],
	out_buf: [u8; 4000],
	tcp: &'a dyn TcpClient<TcpSocket=S, Error=PolkaProviderError>,
}

impl<'a, S> PolkaProvider<'a, S>
{
	pub fn new(tcp: &dyn TcpClient<TcpSocket=S, Error=PolkaProviderError>) -> PolkaProvider<S> {
		let mut sock:S;
		if let Ok(s) = tcp.socket() {
			sock = s
		} else {
			panic!("Unable to create socket");
		}
		PolkaProvider {
			tcp,
			socket: sock,
			ws: ws::WebSocketClient::new_client(rand::thread_rng()),
			in_buf: [0_u8;  4000],
			out_buf: [0_u8;  4000],
		}
	}

	pub fn connect(&mut self, address: &str) -> Result<(), PolkaProviderError> {
		// TODO parse address
		// let's not parse domains for the moment as we are still not able to find the
		// corresponding IP (DNS client)
		let addr = embedded_nal::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9944);

		self.tcp.connect(&mut self.socket, addr)?;

		// initiate a websocket opening handshake
		let websocket_options = WebSocketOptions {
			path: "",
			host: "localhost:9944",
			origin: "http://localhost:9944",
			sub_protocols: None,
			additional_headers: None,
		};
		let (len, web_socket_key) = self.ws.client_connect(&websocket_options, &mut self.out_buf)?;

		let _ = self.tcp.send(&mut self.socket, &self.out_buf[..len])?;
		// if written != len {
		// 	PolkaProviderError::TcpSocket(TcpError::CountNotMatching)
		// }

		// read the response from the server and check it to complete the opening handshake
		let received_size = self.tcp.receive(&mut self.socket, &mut self.in_buf)?;
		self.ws.client_accept(&web_socket_key, &mut self.in_buf[..received_size])?;

		Ok(())
	}

	pub fn disconnect(&mut self) -> Result<(), PolkaProviderError> {
		// initiate a close handshake
		let send_size = self.ws.close(WebSocketCloseStatusCode::NormalClosure, None, &mut self.out_buf)?;
		self.tcp.send(&mut self.socket, &self.out_buf[..send_size])?;

		// read the reply from the server to complete the close handshake
		let received_size = self.tcp.receive(&mut self.socket, &mut self.in_buf)?;
		let ws_result = self.ws.read(&self.in_buf[..received_size], &mut self.out_buf)?;
		match ws_result.message_type {
			WebSocketReceiveMessageType::CloseCompleted => {
				self.tcp.close(&self.socket)?;
				Ok(())
			}
			_ => {
				Err(PolkaProviderError::TcpSocket(TcpError::CannotClose))
			}
		}



	}

	pub fn send(&mut self, message: &str) -> Result<&str, PolkaProviderError> {
		let send_size = self.ws.write(
			WebSocketSendMessageType::Text,
			true,
			message.as_ref(),
			&mut self.out_buf,
		)?;

		self.tcp.send(&mut self.socket, &mut self.out_buf)?;

		// read the response from the server (we expect the server to simply echo the same message back)
		let received_size = self.tcp.receive(&mut self.socket, &mut self.in_buf)?;
		let ws_result = self.ws.read(&self.in_buf[..received_size], &mut self.out_buf)?;
		match ws_result.message_type {
			WebSocketReceiveMessageType::Text => {
				let res = core::str::from_utf8(&self.out_buf[..ws_result.len_to])?;
				Ok(res)
			}
			_ => {
				Err(PolkaProviderError::WebSocket(ws::Error::Unknown))
			}
		}
	}
}
