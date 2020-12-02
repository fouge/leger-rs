#![cfg_attr(not(feature = "std"), no_std)]
#![no_builtins]

use embedded_nal::{TcpClient};
use crate::rpc::{Rpc, RpcError};

mod account;
mod extrinsic;

#[cfg(test)]
mod tests;
mod rpc;

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

	/// Get block genesis hash
	pub fn genesis_hash(&mut self) -> Result<&str, ProviderError> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method(Some("chain_getBlockHash"), Some([0_usize]))?;
		Ok(res)
	}

	pub fn system_version(&mut self) -> Result<&str, ProviderError> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method(Some("system_version"), None)?;
		Ok(res)
	}

	pub fn chain_info(&mut self) -> Result<&str, ProviderError> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method(Some("system_name"), None)?;
		Ok(res)
	}

	pub fn runtime_version(&mut self) -> Result<&str, ProviderError> {
		if !self.rpc.is_connected() {
			self.rpc.connect(self.addr)?;
		}

		let res = self.rpc.rpc_method(Some("state_getRuntimeVersion"), None)?;
		Ok(res)
	}
}
