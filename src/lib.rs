#![cfg_attr(not(feature = "std"), no_std)]
#![no_builtins]

use embedded_nal::{TcpClient};
use crate::rpc::{Rpc, RpcError};
use crate::chain::Chain;

#[cfg(test)]
mod tests;

mod account;
mod extrinsic;
mod rpc;
pub mod chain;

#[derive(Debug)]
pub enum ProviderError {
	RpcError(RpcError)
}

#[derive(Debug)]
pub enum TcpError {
	CountNotMatching,
	CannotCreate,
	CannotClose,
	InvalidAddress,
	Unknown,
}

impl From<RpcError> for ProviderError {
	fn from(err: RpcError) -> ProviderError {
		ProviderError::RpcError(err)
	}
}

pub struct Provider<'a, S> {
	rpc: Rpc<'a, S>,
	addr: &'a str,
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
			addr
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

		let res = self.rpc.rpc_method::<Option<()>>(Some("state_getRuntimeVersion"), None)?;
		Ok(res)
	}
}

impl<S>  Chain for Provider<'_, S> {
	type Error = ProviderError;

	fn get_block_hash(&mut self, number: Option<[usize; 1]>) -> Result<&str, Self::Error> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method(Some("chain_getBlockHash"), number)?;
		Ok(res)
	}

	fn get_genesis_block_hash(&mut self) -> Result<&str, Self::Error> {
		self.get_block_hash(Some([0_usize; 1]))
	}

	fn get_finalized_head(&mut self) -> Result<&str, Self::Error> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method::<Option<()>>(Some("chain_getFinalizedHead"), None)?;
		Ok(res)
	}
}