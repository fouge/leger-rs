use crate::account::Account;
use hex;

/// Test key creation from secret seed.
/// Private and public keys taken from https://substrate.dev/docs/en/knowledgebase/integrate/subkey
#[test]
fn test_new_account() {
	let mut seed:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"ad282c9eda80640f588f812081d98b9a2333435f76ba4ad6258e9c6f4a488363",
		&mut seed as &mut [u8])
		.expect("Cannot decode hex string");
	let mut account_id:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"f6a7ac42a6e1b5329bdb4e63c8bbafa5301add8102843bfe950907bd3969d944",
		&mut account_id as &mut [u8])
		.expect("Cannot decode hex string");

	let mut public = [0_u8; 32];
	let mut private = [0_u8; 64];
	public.copy_from_slice(Account::new(seed).keys.pk.as_ref());
	private.copy_from_slice(Account::new(seed).keys.sk.as_ref());

	assert_eq!(public, account_id);
}

// TODO add test sign_tx