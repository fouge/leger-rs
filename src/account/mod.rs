#[cfg(test)]
mod tests;

use ed25519_compact::{KeyPair, Seed, Noise};

/// Key type is an array of 32 bytes
pub type Key = [u8; 32];

pub struct Account {
	keys: KeyPair, /// Public (account ID) and secret keys are stored into the `KeyPair`
}

impl Account {
	/// Creates an account from private key (secret seed)
	/// Creating account from secret phrase is not supported yet.
	pub fn new(private_key: Key) -> Account {
		// Generates a new key pair using private key as seed.
		let key_pair = KeyPair::from_seed(Seed::new(private_key));

		Account{ keys: key_pair }
	}

	pub fn sign_tx(&self, msg: &mut [u8]) {
		let signed = self.keys.sk.sign(&msg, Some(Noise::default()));
		msg.copy_from_slice(signed.as_ref());
	}
}