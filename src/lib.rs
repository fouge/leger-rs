#![cfg_attr(not(feature = "std"), no_std)]
#![no_builtins]

use embedded_nal::{TcpClient};
use crate::rpc::{Rpc, RpcError};
use crate::chain::Chain;
use crate::extrinsic::{ExtrinsicPayload, ExtrinsicCalls};
use crate::account::{Account, AccountError};

use core::convert::TryFrom;
use core::str::from_utf8;
use crate::scale::Compact;
use crate::calls::Call;
use crate::calls::transfer::ExtrinsicTransferCall;

#[cfg(target_arch = "arm")]
extern crate panic_halt;

#[cfg(test)]
mod tests;

pub mod account;
pub mod chain;
pub mod calls;
pub mod extrinsic;
pub mod scale;
mod rpc;

#[derive(Debug)]
pub enum ProviderError {
	RpcError(RpcError),
	AccountError(AccountError),
	CannotParse,
	InvalidSize,
}

#[derive(Debug)]
pub enum TcpError {
	CountNotMatching,
	CannotConnect,
	CannotClose,
	InvalidAddress,
	Unknown,
}

impl From<RpcError> for ProviderError {
	fn from(err: RpcError) -> ProviderError {
		ProviderError::RpcError(err)
	}
}

impl From<AccountError> for ProviderError {
	fn from(err: AccountError) -> ProviderError {
		ProviderError::AccountError(err)
	}
}

pub struct Provider<'a, S> {
	rpc: Rpc<'a, S>,
	addr: &'a str,
	genesis: Option<[u8; 32]>,
}

impl<'a, S> Provider<'a, S>
{
	/// Creates a provider to connect to a remote Substrate chain.
	/// * Can use any TCP stack implementing [`embedded_nal::TcpClient`](../embedded_nal/trait.TcpClient.html) trait with socket of type `S`.
	/// * Remote address should respect the format: `IP:port`.
	/// * A connection attempt is performed but doesn't yield an error if it fails. Attempts will be made when needed.
	/// ## Errors
	/// * [`ProviderError`](enum.ProviderError.html) returns an [`RpcError`](enum.ProviderError.html#variant.RpcError) if RPC service is not created.
	pub fn new(tcp: &'a dyn TcpClient<TcpSocket=S, Error=TcpError>, addr: &'a str) -> Result<Provider<'a, S>, ProviderError> {
		let mut rpc:Rpc<S>;
		match Rpc::new(tcp) {
			Ok(r) => {
				rpc = r;
			}
			Err(e) => {
				return Err(ProviderError::RpcError(e))
			}
		}

		// try to connect without taking into account if it fails
		let _ = rpc.connect(addr);

		Ok(Provider {
			rpc,
			addr,
			genesis: None,
		})
	}

	pub fn system_version(&mut self) -> Result<&str, ProviderError> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method::<Option<()>>(Some("system_version"), None)?;
		Ok(res)
	}

	pub fn system_name(&mut self) -> Result<&str, ProviderError> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method::<Option<()>>(Some("system_name"), None)?;
		Ok(res)
	}

	pub fn runtime_version(&mut self) -> Result<&str, ProviderError> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		// response:
		// {"jsonrpc":"2.0","result":{"apis":[["0xdf6acb689907609b",3],["0x37e397fc7c91f5e4",1],["0x40fe3ad401f8959a",4],["0xd2bc9897eed08f15",2],["0xf78b278be53f454c",2],["0xdd718d5cc53262d4",1],["0xab3c0572291feb8b",1],["0xed99c5acb25eedf5",2]],"authoringVersion":1,"implName":"node-template","implVersion":1,"specName":"node-template","specVersion":1,"transactionVersion":1},"id":3}
		let res = self.rpc.rpc_method::<Option<()>>(Some("state_getRuntimeVersion"), None)?;
		Ok(res)
	}
}

impl<S>  Chain for Provider<'_, S> {
	type Error = ProviderError;

	fn get_block_hash(&mut self, number: Option<[usize; 1]>) -> Result<[u8; 32], Self::Error> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method(Some("chain_getBlockHash"), number)?;
		let mut block_hash = [0_u8; 32];

		let hex_data = res.strip_prefix("0x").map_or(
			res,
			|v| v
		);

		if hex::decode_to_slice(
			hex_data,
			&mut block_hash as &mut [u8]).is_err() {
			return Err(ProviderError::CannotParse)
		}

		Ok(block_hash)
	}

	fn get_genesis_block_hash(&mut self) -> Result<[u8; 32], Self::Error> {
		if let Some(g) = self.genesis {
			return Ok(g)
		}

		let genesis = match self.get_block_hash(Some([0_usize; 1])) {
			Ok(g) => {g}
			Err(e) => {
				return Err(e)
			}
		};

		self.genesis.replace(genesis);

		self.genesis.ok_or(ProviderError::CannotParse)
	}

	fn get_finalized_head(&mut self) -> Result<&str, Self::Error> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method::<Option<()>>(Some("chain_getFinalizedHead"), None)?;
		Ok(res)
	}
}


/// Methods param has a few fields
/// | header (= "0x" + payload size) | payload (extrinsic struct) |
/// `header` contains string "0x" followed by `payload` size as SCALE encoded hex string

/// Maximum header size
const MAXIMUM_HEADER_SIZE_BYTES: usize = 8;

/// Maximum payload size
const MAXIMUM_PAYLOAD_SIZE_BYTES_ASCII: usize = 504;
const MAXIMUM_PAYLOAD_SIZE_BYTES: usize = 504/2;

/// Maximum method's param size in bytes
const MAXIMUM_PARAM_SIZE_BYTES: usize = MAXIMUM_HEADER_SIZE_BYTES + MAXIMUM_PAYLOAD_SIZE_BYTES_ASCII;


impl<S> ExtrinsicCalls for Provider<'_, S> {
	type Error = ProviderError;

	/// This function is trying to be as memory-efficient as possible by using only one buffer
	/// to get the payload and translating it in hex characters
	/// The size of this buffer is `MAXIMUM_PARAM_SIZE_BYTES`
	///
	/// ## Errors
	/// * `AccountError::*`: Impossible to fetch source account info
	/// * `InvalidSize`: Error with buffer size and payload size (buffer isn't large enough?)
	/// * `RpcError::*`: Error sending the RPC request `author_submitExtrinsic`.
	fn submit_extrinsic(&mut self, author: &mut Account, method: &dyn Call) -> Result<&str, Self::Error> {
		let nonce;
		if let Ok(n) = author.get_nonce(self) {
			nonce = n;
		} else {
			nonce = 0;
		}

		let extrinsic = ExtrinsicPayload::new(self, method, nonce)?;

		let mut param_buf = [0_u8; MAXIMUM_PARAM_SIZE_BYTES];
		param_buf[0] = 0x30; // "0"
		param_buf[1] = 0x78; // "x"

		let payload_size;
		{
			let mut sig_payload:&mut [u8; MAXIMUM_PAYLOAD_SIZE_BYTES] =
				<&mut [u8; MAXIMUM_PAYLOAD_SIZE_BYTES]>::try_from(&mut param_buf[MAXIMUM_HEADER_SIZE_BYTES..MAXIMUM_HEADER_SIZE_BYTES+MAXIMUM_PAYLOAD_SIZE_BYTES]).unwrap();

			payload_size = extrinsic.signed_tx(author, &mut sig_payload);
		}

		if payload_size != 0_usize {
			// append payload size as a header (scale compacted)
			let size_u32 = payload_size as u32;
			let mut buf = [0_u8; MAXIMUM_HEADER_SIZE_BYTES-2];
			let size_header_length = size_u32.scale_compact(&mut buf);

			let header:&mut [u8; MAXIMUM_HEADER_SIZE_BYTES] = <&mut [u8; MAXIMUM_HEADER_SIZE_BYTES]>::try_from(&mut param_buf[..MAXIMUM_HEADER_SIZE_BYTES]).unwrap();
			match size_header_length {
				2 => {
					let size_bytes: &mut[u8; 2] = <&mut [u8; 2]>::try_from(&mut buf[..2]).unwrap();
					hex::encode_to_slice::<[u8; 2]>(*size_bytes, &mut header[MAXIMUM_HEADER_SIZE_BYTES-4..MAXIMUM_HEADER_SIZE_BYTES].as_mut()).unwrap();
				}
				_ => {}
			}

			{
				let sig_payload:&mut [u8; MAXIMUM_PAYLOAD_SIZE_BYTES] = <&mut [u8; MAXIMUM_PAYLOAD_SIZE_BYTES]>::try_from(&mut param_buf[MAXIMUM_HEADER_SIZE_BYTES..MAXIMUM_HEADER_SIZE_BYTES+ MAXIMUM_PAYLOAD_SIZE_BYTES]).unwrap();
				hex::encode_to_slice::<[u8; MAXIMUM_PAYLOAD_SIZE_BYTES_ASCII /2]>(*sig_payload, &mut param_buf[MAXIMUM_HEADER_SIZE_BYTES..]).unwrap();
			}

			let res;
			{
				let start_idx = MAXIMUM_HEADER_SIZE_BYTES-size_header_length*2-2;
				param_buf[start_idx] = 0x30; // "0"
				param_buf[start_idx+1] = 0x78; // "x"
				let sig_payload_str:&str = from_utf8(param_buf[start_idx..MAXIMUM_HEADER_SIZE_BYTES+payload_size*2].as_ref()).unwrap();
				res = self.rpc.rpc_method(Some("author_submitExtrinsic"), Some([sig_payload_str]))?;
			}
			Ok(res)
		} else {
			Err(ProviderError::InvalidSize)
		}
	}

	/// This function creates the Call object to transfer balance between author and `dest_account`
	/// And then submit the extrinsic
	fn balance_transfer(&mut self, author: &mut Account, dest_account: &[u8; 32], amount: u128)
						-> Result<&str, Self::Error> {
		let method = ExtrinsicTransferCall::new(dest_account, amount);

		self.submit_extrinsic(author, &method)
	}
}