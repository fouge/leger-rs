use embedded_websocket as ws;
use embedded_websocket::{WebSocketOptions, WebSocketSendMessageType, WebSocketReceiveMessageType, WebSocketCloseStatusCode};
use embedded_nal::{TcpClientStack};
use rand::rngs::ThreadRng;
use core::str::FromStr;
use serde::{Serialize, Deserialize};
use heapless::{String, consts::*};
use crate::TcpError;

#[derive(Debug)]
pub enum JsonError {
	ErrorParsing,
	ErrorCode(i64)
}

#[derive(Debug)]
pub enum RpcError {
	WebSocket(ws::Error),
	TcpSocket(TcpError),
	Embedded(embedded_nal::nb::Error<TcpError>),
	Json(JsonError),
	ResponseDoesNotMatch,
	ErrorClosing,
	Utf8Error,
	Unknown
}

impl From<ws::Error> for RpcError {
	fn from(err: ws::Error) -> RpcError {
		RpcError::WebSocket(err)
	}
}

impl From<TcpError> for RpcError {
	fn from(err: TcpError) -> RpcError {
		RpcError::TcpSocket(err)
	}
}

impl From<embedded_nal::nb::Error<TcpError>> for RpcError {
	fn from(err: embedded_nal::nb::Error<TcpError>) -> RpcError {
		RpcError::Embedded(err)
	}
}

impl From<JsonError> for RpcError {
	fn from(err: JsonError) -> RpcError {
		RpcError::Json(err)
	}
}

impl From<core::str::Utf8Error> for RpcError {
	fn from(_: core::str::Utf8Error) -> RpcError {
		RpcError::Utf8Error
	}
}

impl From<embedded_nal::nb::Error<RpcError>> for RpcError {
	fn from(err: embedded_nal::nb::Error<RpcError>) -> RpcError {
		if let embedded_nal::nb::Error::Other(e) = err {
			e
		} else {
			RpcError::Unknown
		}
	}
}

pub struct Rpc<'a, S> {
	socket: S,
	ws: ws::WebSocketClient<ThreadRng>,
	in_buf: [u8; 4096],
	out_buf: [u8; 4096],
	tcp: &'a dyn TcpClientStack<TcpSocket=S, Error=TcpError>,
	cmd_id: usize,
}

#[derive(Serialize, Deserialize)]
struct JsonRpc<'a, T> {
	id: usize,
	jsonrpc: &'a str,
	#[serde(skip_serializing_if = "Option::is_none")]
	method: Option<&'a str>,
	#[serde(skip_serializing_if = "Option::is_none")]
	result: Option<&'a str>,
	#[serde(skip_serializing_if = "Option::is_none")]
	params: Option<T>,
}

#[derive(Serialize, Deserialize)]
struct ErrorCode <'a> {
	#[serde(skip_serializing_if = "Option::is_none")]
	code: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	message: Option<&'a str>,
}

//"{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32600,\"message\":\"Invalid request\"},\"id\":7}"
#[derive(Serialize, Deserialize)]
struct JsonErrorResponse<'a> {
	id: usize,
	jsonrpc: &'a str,
	#[serde(skip_serializing_if = "Option::is_none")]
	error: Option<ErrorCode<'a>>
}

impl<'a, S> Rpc<'a, S>
{
	/// Instantiates the provider and init TCP socket, websocket lib and static buffers.
	///
	/// # Errors
	/// * `TcpError::CannotCreate` if the TCP socket cannot be created
	pub fn new(tcp: &dyn TcpClientStack<TcpSocket=S, Error=TcpError>) -> Result<Rpc<S>, RpcError> {
		let sock: S;
		if let Ok(s) = tcp.socket() {
			sock = s
		} else {
			return Err(RpcError::TcpSocket(TcpError::CannotCreate))
		}

		Ok(Rpc {
			tcp,
			socket: sock,
			ws: ws::WebSocketClient::new_client(rand::thread_rng()),
			in_buf: [0_u8; 4096],
			out_buf: [0_u8; 4096],
			cmd_id: 1_usize,
		})
	}

	/// Connects to the node at the given address. Initiates the websocket handshake.
	///
	/// # Errors
	/// * `embedded_websocket::Error`: if any error with websocket
	/// * `TcpError::InvalidAddress`: address cannot be parsed
	/// * `TcpError::CountNotMatching`: sent bytes count doesn't equal the initial packet count
	pub fn connect(&mut self, address: &str) -> Result<(), RpcError> {
		// TCP connection first
		if let Ok(addr) = embedded_nal::SocketAddr::from_str(address) {
			self.tcp.connect(&mut self.socket, addr)?;
		} else {
			return Err(RpcError::TcpSocket(TcpError::InvalidAddress))
		}

		// initiate a websocket opening handshake
		let websocket_options = WebSocketOptions {
			path: "",
			host: "localhost:9944",
			origin: "http://localhost:9944",
			sub_protocols: None,
			additional_headers: None,
		};
		let (len, web_socket_key) = self.ws.client_connect(&websocket_options, &mut self.out_buf)?;

		// send websocket frame using tcp socket
		let written = self.tcp.send(&mut self.socket, &self.out_buf[..len])?;
		if written != len {
			return Err(RpcError::TcpSocket(TcpError::CountNotMatching))
		}

		// read the response from the server and check it to complete the opening handshake
		let received_size = self.tcp.receive(&mut self.socket, &mut self.in_buf)?;
		self.ws.client_accept(&web_socket_key, &mut self.in_buf[..received_size])?;

		Ok(())
	}

	/// Returns TCP socket state
	pub fn is_connected(&self) -> bool {
		if let Ok(c) = self.tcp.is_connected(&self.socket) {
			c
		} else {
			false
		}
	}

	/// Disconnects from the node by initiating a close handshake.
	/// The TCP socket will be closed when the `PolkaProvider` instance is dropped.
	///
	/// # Errors
	/// * `ErrorClosing` if the WebSocket has not been closed properly.
	pub fn disconnect(&mut self) -> Result<(), RpcError> {
		// initiate a close handshake
		let send_size = self.ws.close(WebSocketCloseStatusCode::NormalClosure, None, &mut self.out_buf)?;
		self.tcp.send(&mut self.socket, &self.out_buf[..send_size])?;

		// read the reply from the server to complete the close handshake
		let received_size = self.tcp.receive(&mut self.socket, &mut self.in_buf)?;
		let ws_result = self.ws.read(&self.in_buf[..received_size], &mut self.out_buf)?;
		match ws_result.message_type {
			WebSocketReceiveMessageType::CloseCompleted => {
				// we can close the TCP socket as well
				self.tcp.close(&self.socket)?;
				Ok(())
			}
			_ => {
				Err(RpcError::ErrorClosing)
			}
		}
	}

	/// Send request with response (blocking wait)
	fn request(&mut self, message: &str) -> Result<&str, RpcError> {
		// create WS frame with message argument as payload
		let len = self.ws.write(
			WebSocketSendMessageType::Text,
			true,
			message.as_ref(),
			&mut self.out_buf,
		)?;

		// send websocket frame
		let written = self.tcp.send(&mut self.socket, &mut self.out_buf[..len])?;
		if len != written {
			return Err(RpcError::TcpSocket(TcpError::CountNotMatching))
		}

		// read the response from the server and parse websocket message
		let received_size = self.tcp.receive(&mut self.socket, &mut self.in_buf)?;
		let ws_result = self.ws.read(&self.in_buf[..received_size], &mut self.out_buf)?;
		match ws_result.message_type {
			WebSocketReceiveMessageType::Text => {
				let res = core::str::from_utf8(&self.out_buf[..ws_result.len_to])?;
				Ok(res)
			}
			WebSocketReceiveMessageType::CloseMustReply => {
				// Signals that the other party has initiated the close handshake. If you receive this
				// message you should respond with a `WebSocketSendMessageType::CloseReply` with the
				// same payload as close message
				// TODO not tested
				let len = self.ws.write(
					WebSocketSendMessageType::CloseReply,
					true,
					&self.out_buf[..ws_result.len_to], // take payload from received message
					&mut self.in_buf,
				)?;
				self.tcp.send(&mut self.socket, &mut self.in_buf[..len])?;

				Err(RpcError::WebSocket(ws::Error::Unknown))
			}
			_ => {
				Err(RpcError::WebSocket(ws::Error::Unknown))
			}
		}
	}

	/// Call rpc method with optional params
	/// Field `result` is returned from the response if it can be parsed as a string
	/// Otherwise, the whole JSON response is returned.
	///
	/// # Errors
	/// * `ResponseDoesNotMatch`: JSON returned has been parsed but returned `id` is not the same as
	/// the sent `id`
	/// * any other error than can happen with `request()`
	pub fn rpc_method<T: Serialize>(&mut self, method: Option<&str>, params: Option<T>) -> Result<&str, RpcError> {
		// construct request from method and params
		let json_req = JsonRpc {
			id: self.cmd_id,
			jsonrpc: "2.0",
			method: method,
			params,
			result: None
		};
		let req_str: String<U512> = serde_json_core::to_string(&json_req).unwrap();
		let req_str = req_str.as_str();
		self.cmd_id = self.cmd_id + 1_usize;
		let response = self.request(req_str);

		// Parse response if it contains a result string
		// returns the whole response if JSON cannot be parsed
		match response {
			Ok(res) => {
				if let Ok(json_res) = serde_json_core::from_str::<JsonRpc<Option<&str>>>(res) {
					if json_res.id == json_req.id {
						if let Some(result) = json_res.result {
							return Ok(result)
						} else {
							if let Ok(json_err) = serde_json_core::from_str::<JsonErrorResponse>(res) {
								if let Some(error) = json_err.error {
									if let Some(code) = error.code {
										return Err(RpcError::Json(JsonError::ErrorCode(code)))
									}
								}
							}
						}
						Err(RpcError::Json(JsonError::ErrorParsing))
					} else {
						Err(RpcError::ResponseDoesNotMatch)
					}
				} else {
					// The response is not a JsonRpc,
					// at the moment, let's return the whole response struct
					// to better analyze the result
					Ok(res)
				}
			}
			Err(e) => {
				Err(e)
			}
		}
	}
}
