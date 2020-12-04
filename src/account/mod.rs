#[cfg(test)]
mod tests;

use ed25519_compact::{KeyPair, Seed, Noise, PublicKey};
use crate::Provider;
use core::{str, mem};
use heapless::{String, Vec, consts::*};
use blake2_rfc::blake2b::Blake2b;
use core::convert::TryInto;

#[derive(Debug)]
pub enum AccountError {
	CannotFetchAccountInfo,
	CannotConvert,
}

#[repr(C)]
#[derive(Debug)]
pub struct AccountInfo {
	nonce: u32,
	ref_count: u32,
	data: AccountData
}

#[repr(C)]
#[derive(Debug)]
pub struct AccountData {
	free: u128,
	reserved: u128,
	misc_frozen: u128,
	free_frozen: u128,
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

impl KeyFormat for PublicKey {
	fn to_ss58(&self) -> String<U64> {
		let mut body = [0_u8; 35];
		let mut output = [0_u8; 64];

		// concatenate address type and public key
		// address-Type is Generic Substrate wildcard
		body[0] = 0x2A;
		body[1..].iter_mut()
			.zip(self.iter())
			.for_each(|(f, t)| *f = *t);

		let mut hasher = Blake2b::new(64);
		hasher.update(PREFIX);
		hasher.update(body[0..33].as_ref());
		let hash = hasher.finalize();

		body[33..].iter_mut().zip(hash.as_ref().iter())
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

	pub fn u8a(&self) -> Key {
		*self.keys.pk
	}

	/// Get account info from node storage
	/// ## Errors
	/// * CannotConvert: there has been an error converting between: slice <-> hex str
	/// *
	pub fn get_info<S>(&mut self, provider: &mut Provider<S>) -> Result<AccountInfo, AccountError> {
		// The request is a concatenation as hex string of:
		//  - key (System, Account) xxhashes
		//  - Account ID blake2b (16 bytes)
		//	- Account ID (transparent)
		const REQ_SIZE: usize = 80;
		let mut params = [0_u8; REQ_SIZE];

		// "System".xxHash128 = "26AA394EEA5630E07C48AE0C9558CEF7"
		// "Account".xxHash128 = "B99D880EC681799C0CF30E8886371DA9"
		if hex::decode_to_slice(
			"26AA394EEA5630E07C48AE0C9558CEF7B99D880EC681799C0CF30E8886371DA9",
			&mut params[0..32]).is_err() {
			return Err(AccountError::CannotConvert)
		}

		// We need to hash the public key into Blake2b (16 bytes)
		let u8a = self.u8a();

		let mut hasher = Blake2b::new(16);
		hasher.update(u8a.as_ref());
		let hash = hasher.finalize();

		// Copy hash and account ID value into the `params` array
		params[32..].iter_mut()
			.zip(hash.as_ref().iter())
			.for_each(|(t, f)| *t = *f);

		params[48..].iter_mut()
			.zip(u8a.iter())
			.for_each(|(t, f)| *t = *f);

		// `params` has been filled
		// we can encode as hex string
		let mut enc_dec_buf = [0_u8; REQ_SIZE*2+2];
		enc_dec_buf[0] = 0x30; // "0"
		enc_dec_buf[1] = 0x78; // "x"
		hex::encode_to_slice(params, &mut enc_dec_buf[2..]).unwrap();

		let v: Vec<u8, U256> = Vec::from_slice(enc_dec_buf.as_ref()).unwrap();
		let s: String<U256> = String::from_utf8(v).unwrap();

		// Sending the RPC request
		let rpc_response = provider.rpc.rpc_method(Some("state_getStorage"), Some([s.as_str()]));

		// AccountInfo is packed into an hex string starting with "0x".
		// Let's parse if
		if let Ok(r) = rpc_response {
			let hex_data = r.strip_prefix("0x").map_or(
				r,
				|v| v
			);

			// Now that we have removed 0x, we can parse the hex string into a slice
			// so we can unpack into AccountInfo
			if hex::decode_to_slice(hex_data, &mut enc_dec_buf[..hex_data.len()/2]).is_ok() {
				let acc;
				unsafe { acc = mem::transmute::<[u8; 72], AccountInfo>(enc_dec_buf[0..72].try_into().expect("Cannot convert slice to array")); }
				return Ok(acc)
			} else {
				return Err(AccountError::CannotConvert)
			}
		}

		Err(AccountError::CannotFetchAccountInfo)
	}

	pub fn get_balance<S>(&mut self, provider: &mut Provider<S>) -> Result<u128, AccountError> {
		let info = self.get_info(provider)?;
		Ok(info.data.free)
	}
}