use crate::account::Account;
use crate::scale::Compact;
use crate::{ProviderError, MAXIMUM_PAYLOAD_SIZE_BYTES};
use crate::chain::Chain;
use crate::calls::Call;

pub enum ExtrinsicEra {
	Immortal,
	Mortal,
}


pub struct ExtrinsicPayload<'a> {
	method: &'a dyn Call,
	era: [u8; 1], // immortal: 0x00
	nonce: u32, // SCALE encoded
	tip: u128, // SCALE encoded
	spec_version: u32,
	transaction_version: u32,
	genesis: [u8; 32],
	block_hash: [u8; 32],
}

pub trait ExtrinsicCalls {
	type Error: core::fmt::Debug;

	fn submit_extrinsic(&mut self, author: &mut Account, method: &dyn Call) -> Result<&str, Self::Error>;

	fn balance_transfer(&mut self, author: &mut Account, dest_account: &[u8; 32], amount: u128)
						-> Result<&str, Self::Error>;
}

impl<'a> ExtrinsicPayload<'a> {
	/// Creates a new `ExtrinsicPayload` structure to be serialized.
	///
	/// ## Errors
	/// Initialization of the structure needs access to the blockchain to get genesis and block hash,
	/// if those have not been fetched before.
	/// Thus, creating an extrinsic can return an error; see `ProviderError`
	pub fn new(chain: &mut dyn Chain<Error=ProviderError>, call: &'a dyn Call, nonce: u32) -> Result<ExtrinsicPayload<'a>, ProviderError> {
		let genesis = chain.get_genesis_block_hash()?;
		let block_hash = chain.get_block_hash(None)?;
		//let transaction_version =
		Ok(ExtrinsicPayload {
			method: call,
			era: [0x00], // immortal TODO implement Mortal era
			nonce,
			tip: 0,
			spec_version: 1, // TODO get from `runtime_version`
			transaction_version: 1,  // TODO get from `runtime_version`
			genesis,
			block_hash
		})
	}

	/// Generates the signature payload used to compute a signature
	/// The call block located at the beginning of the sig payload is to be used in the
	/// extrinsic payload.
	/// Thus, two sizes are returned: the call size and the signature payload size
	fn signature_payload(&self, payload: &mut [u8]) -> (usize, usize) {
		// method
		let mut idx = self.method.encode(&mut payload[0..]);
		let call_size = idx;

		// era
		payload[idx] = self.era[0];
		idx += 1;

		// nonce
		let nonce = self.nonce as u128;
		idx += nonce.scale_compact(&mut payload[idx..]);

		// tip: `Balance` used to prioritize transaction, let's keep it 0
		let tip = 0_u128;
		idx += tip.scale_compact(&mut payload[idx..]);

		// spec version
		let mut u32_buf = self.spec_version.to_le_bytes();
		payload[idx..idx+u32_buf.len()].copy_from_slice(u32_buf.as_ref());
		idx += u32_buf.len();

		// transaction version
		u32_buf = self.transaction_version.to_le_bytes();
		payload[idx..idx+u32_buf.len()].copy_from_slice(u32_buf.as_ref());
		idx += u32_buf.len();

		// genesis hash
		payload[idx..idx+self.genesis.len()].copy_from_slice(self.genesis.as_ref());
		idx += self.genesis.len();

		// hash of the “checkpoint block”, which is to say the first block of the era specified
		// by the era field. If just making the transaction “immmortal”, then the genesis hash
		// of the blockchain should be used.
		if self.era[0] == 0 {
			payload[idx..idx+self.genesis.len()].copy_from_slice(self.genesis.as_ref());
			idx += self.genesis.len();
		} else {
			unimplemented!();
		}

		(call_size, idx)
	}

	/// Generates the extrinsic payload to be sent and put it into `signed_tx`.
	/// The actual size of the payload is returned.
	/// Payload is signed using the account `sender_account`.
	///
	/// ## Errors
	/// * returns `0` if `signed_tx` buffer is not large enough
	pub fn signed_tx(&self, sender_account: &Account, signed_tx: &mut [u8; MAXIMUM_PAYLOAD_SIZE_BYTES]) -> usize {
		// we keep the packed call (module index, call index & params) in a temporary buffer
		// as we need it in the final payload
		let mut temp_packed_call = [0_u8; 64];

		// compose the extrinsic payload that is about to be signed
		let (packed_call_size, payload_size) = self.signature_payload(signed_tx.as_mut());

		// sign the payload
		let mut signature = [0_u8; 64];
		sender_account.sign_tx(signed_tx[..payload_size].as_mut(), &mut signature);

		// copy the `call` part to be sent along with the extrinsic signature
		temp_packed_call[..packed_call_size].copy_from_slice(signed_tx[..packed_call_size].as_ref());

		signed_tx[0] = 0x84;
		// signed_tx[1] = 0xFF;

		let mut idx = 1_usize;

		signed_tx[idx..].iter_mut().zip(sender_account.u8a().iter())
			.for_each(|(t, f)| *t = *f);
		idx += sender_account.u8a().len();

		signed_tx[idx] = 0x01;
		idx += 1;

		signed_tx[idx..].iter_mut().zip(signature.iter())
			.for_each(|(t, f)| *t = *f);
		idx += signature.len();

		// era, immortal
		signed_tx[idx] = self.era[0];
		idx += 1;

		idx += self.nonce.scale_compact(&mut signed_tx[idx..]);
		idx += self.tip.scale_compact(&mut signed_tx[idx..]);

		// append packed call
		signed_tx[idx..idx+packed_call_size].copy_from_slice(temp_packed_call[..packed_call_size].as_ref());
		idx += packed_call_size;

		return if idx < signed_tx.len() {
			idx
		} else {
			0
		}
	}

}

