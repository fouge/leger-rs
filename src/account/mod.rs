#![cfg_attr(not(feature = "std"), no_std)]
#![no_builtins]

#[cfg(test)]
mod tests;

use ed25519_compact::{KeyPair, Seed, Noise};
use crate::Provider;
use core::str;
use heapless::{String, Vec, consts::*};
use serde::de::value::U64Deserializer;
use blake2::{Blake2b, Blake2s, Digest};

enum AccountError {
	CannotFetchAccountInfo,
	CannotConvert,
}

pub struct AccountInfo {
	nonce: u64,
	ref_count: u64,
	data: AccountData
}

pub struct AccountData {
	free: u64,
	reserved: u64,
	misc_frozen: u64,
	free_frozen: u64,
}

pub struct Account {
	/// Public (account ID) and secret keys are stored into the `KeyPair`
	keys: KeyPair,
	info: Option<AccountInfo>,
	ss58: String<U64>,
}

/// Key type is an array of 32 bytes
pub type Key = [u8; 32];

const PREFIX: &[u8] = b"SS58PRE";

pub trait KeyFormat {
	fn to_ss58(&self) -> String<U64>;
}

impl KeyFormat for Key {
	fn to_ss58(&self) -> String<U64> {
		let mut body = [0_u8; 35];
		let mut output = [0_u8; 64];

		// concatenate address type and public key
		// address-Type is Generic Substrate wildcard
		body[0] = 0x2A;
		body[1..].iter_mut()
			.zip(self.iter())
			.for_each(|(f, t)| *f = *t);

		let mut hasher = Blake2b::new();
		hasher.update(PREFIX);
		hasher.update(body[0..33].as_ref());
		let hash = hasher.finalize();

		body[33..].iter_mut().zip(hash.iter())
			.for_each(|(f, t)| *f = *t);

		let l = bs58::encode(body.as_ref()).into(&mut output[..]).unwrap();
		let v: Vec<u8, U64> = Vec::from_slice(output[..l].as_ref()).unwrap();
		let s: String<U64> = String::from_utf8(v).unwrap();
		s
	}
}

impl Account {
	/// Creates an account from private key (secret seed)
	/// Creating account from secret phrase is not supported yet.
	pub fn new(private_key: Key) -> Account {
		// Generates a new key pair using private key as seed.
		let key_pair = KeyPair::from_seed(Seed::new(private_key));

		let ss58 = key_pair.pk.to_ss58();

		Account { keys: key_pair, info: None, ss58 }
	}

	pub fn sign_tx(&self, msg: &mut [u8]) {
		let signed = self.keys.sk.sign(&msg, Some(Noise::default()));
		msg.copy_from_slice(signed.as_ref());
	}

	pub fn ss58(&self) -> &str {
		self.ss58.as_str()
	}

	pub fn to_u8a(&self) -> &str {
		"not implemented"
	}

	fn get_info<S>(&mut self, provider: &mut Provider<S>) -> Result<AccountInfo, AccountError> {
		provider.rpc.rpc_method(Some("state_getStorage"), Some("0xNone"));

		Err(AccountError::CannotFetchAccountInfo)
	}

	fn get_balance<S>(&mut self, provider: &mut Provider<S>) -> Option<u64> {
		//self.get_balance(provider)?;
		Some(0_u64)
	}
}