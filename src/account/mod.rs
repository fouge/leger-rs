use crate::Provider;
use core::{str, mem};
use heapless::{String, Vec, consts::*};
use blake2_rfc::blake2b::Blake2b;
use core::convert::TryInto;
use core::convert::TryFrom;

#[derive(Debug)]
pub enum AccountError {
	CannotFetchAccountInfo,
	CannotConvert,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AccountInfo {
	nonce: u32,
	ref_count: u32,
	data: AccountData
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AccountData {
	free: u128,
	reserved: u128,
	misc_frozen: u128,
	free_frozen: u128,
}

pub struct Account<'a> {
	public: Key,
	signer: &'a dyn LegerSigner,
	info: Option<AccountInfo>,
}

/// This trait must be implemented depending on hardware specifications.
/// Signing with private key (Ed25519 or Sr25519) should be performed in a secure context
pub trait LegerSigner {
	fn get_public(&self) -> Key;
	fn sign(&self, payload: &[u8], signature: &mut [u8; 64]);
}

/// Key type is an array of 32 bytes
pub type Key = [u8; 32];

pub const PREFIX: &[u8] = b"SS58PRE";

impl<'a> Account<'a> {
	/// Creates an account from private key (secret seed)
	/// Creating account from secret phrase is not supported yet.
	pub fn new(signer: &dyn LegerSigner) -> Account {
		let public = signer.get_public();
		Account { public, signer, info: None }
	}

	/// Generate signature for payload and write it back into the payload (64 bytes)
	///
	/// TODO make this async in case the hardware needs to compute on separate CPU/secure element.
	/// TODO return Result in case of error
	pub fn sign_tx(&self, msg: &mut [u8], signature: &mut [u8; 64]) {
		self.signer.sign(msg, signature);
	}

	/// Get public key array
	pub fn u8a(&self) -> Key {
		self.public
	}

	/// Get account info from node storage.
	/// If the provider is not able to fetch data, the last known data is used.
	///
	/// ## Errors
	/// * CannotConvert: there has been an error converting between: slice <-> hex str
	/// * CannotFetchAccountInfo: error connecting to the provider
	pub fn get_info<S>(&mut self, provider: &mut Provider<S>) -> Result<AccountInfo, AccountError> {
		// The request is a concatenation as hex string of:
		//  - key (System, Account) xxhashes
		//  - Account ID blake2b (16 bytes)
		//	- Account ID (transparent)
		const REQ_SIZE: usize = 80;
		const REQ_SIZE_STR_ENCODED: usize = 2+REQ_SIZE*2; // "0x" + string encoded REQ_SIZE buffer
		const REQ_KEY_HASH_SIZE: usize = 32;
		const REQ_ACCOUNT_B2B_SIZE: usize = 16;
		let mut params = [0_u8; REQ_SIZE_STR_ENCODED];
		params[0] = 0x30; // "0"
		params[1] = 0x78; // "x"

		// In the following block, params[2..] is filled with keys and account ID, encoded as hex string
		{
			// params_to_encode is a subarray of params
			let params_to_encode: &mut[u8; REQ_SIZE] = <&mut [u8; REQ_SIZE]>::try_from(&mut params[2..2+REQ_SIZE]).unwrap();

			// "System".xxHash128 = "26AA394EEA5630E07C48AE0C9558CEF7"
			// "Account".xxHash128 = "B99D880EC681799C0CF30E8886371DA9"
			let hashed_request: [u8; REQ_KEY_HASH_SIZE] = [0x26, 0xAA, 0x39, 0x4E, 0xEA, 0x56, 0x30, 0xE0, 0x7C, 0x48, 0xAE, 0x0C, 0x95, 0x58, 0xCE, 0xF7, 0xB9, 0x9D, 0x88, 0x0E, 0xC6, 0x81, 0x79, 0x9C, 0x0C, 0xF3, 0x0E, 0x88, 0x86, 0x37, 0x1D, 0xA9];
			params_to_encode[..REQ_KEY_HASH_SIZE].copy_from_slice(hashed_request.as_ref());

			// We need to hash the public key into Blake2b (16 bytes)
			let u8a = self.u8a();

			// TODO should we store the hash in Account so that we don't need to compute each time?
			let mut hasher = Blake2b::new(REQ_ACCOUNT_B2B_SIZE);
			hasher.update(u8a.as_ref());
			let hash = hasher.finalize();

			// Copy hash and account ID
			params_to_encode[REQ_KEY_HASH_SIZE..REQ_KEY_HASH_SIZE+REQ_ACCOUNT_B2B_SIZE].copy_from_slice(hash.as_ref());
			params_to_encode[REQ_KEY_HASH_SIZE+REQ_ACCOUNT_B2B_SIZE..].copy_from_slice(u8a.as_ref());

			// `params_to_encode` has been filled
			// we can encode as hex string and copy into params (overwriting params_to_encode)
			hex::encode_to_slice::<[u8; REQ_SIZE]>(*params_to_encode, &mut params[2..REQ_SIZE_STR_ENCODED]).unwrap();
		}

		let s = core::str::from_utf8(params.as_ref()).expect("Cannot convert payload");

		// Sending the RPC request
		let rpc_response = provider.rpc.rpc_method(Some("state_getStorage"), Some([s]));

		// AccountInfo is packed into an hex string starting with "0x".
		// Let's parse it if we have an answer
		// otherwise, use last known AccountInfo
		if let Ok(r) = rpc_response {
			let hex_data = r.strip_prefix("0x").map_or(
				r,
				|v| v
			);

			// Now that we have removed 0x, we can parse the hex string into a slice
			// so we can unpack into AccountInfo
			if hex::decode_to_slice(hex_data, &mut params[..hex_data.len()/2]).is_ok() {
				let acc;
				unsafe { acc = mem::transmute::<[u8; 72], AccountInfo>(params[0..72].try_into().expect("Cannot convert slice to array")); }

				// replace last known account info
				self.info.replace(acc);

				return Ok(acc)
			} else {
				return Err(AccountError::CannotConvert)
			}
		} else if let Some(i) = &self.info {
			Ok(*i)
		} else {
			Err(AccountError::CannotFetchAccountInfo)
		}
	}

	/// Get account balance.
	/// If the provider is not able to fetch data, the last known data is used.
	///
	/// ## Errors
	/// * CannotConvert: there has been an error converting between: slice <-> hex str
	/// * CannotFetchAccountInfo: error connecting to the provider
	pub fn get_balance<S>(&mut self, provider: &mut Provider<S>) -> Result<u128, AccountError> {
		let info = self.get_info(provider)?;
		Ok(info.data.free)
	}

	/// Get account nonce.
	/// If the provider is not able to fetch data, the last known data is used.
	///
	/// ## Errors
	/// * CannotConvert: there has been an error converting between: slice <-> hex str
	/// * CannotFetchAccountInfo: error connecting to the provider
	pub fn get_nonce<S>(&mut self, provider: &mut Provider<S>) -> Result<u32, AccountError> {
		let info = self.get_info(provider)?;
		Ok(info.nonce)
	}
}