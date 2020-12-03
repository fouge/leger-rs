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

	let account = Account::new(seed);
	public.copy_from_slice(account.keys.pk.as_ref());
	private.copy_from_slice(account.keys.sk.as_ref());

	assert_eq!(public, account_id);
}

#[test]
fn test_new_account_2() {
	// mnemonic
	// smoke key grief belt gather absurd open attend keep flip hollow popular

	let mut seed:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"DA5CE9BB3618B9004F0D76C0FE97DA6C075AE60937FC7B3A8C01A16A655E9388",
		&mut seed as &mut [u8])
		.expect("Cannot decode hex string");
	let mut account_id:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"B10355577E96846A5C2144BCC37108BAD78538CB7D11EA0456D041190A19A0B7",
		&mut account_id as &mut [u8])
		.expect("Cannot decode hex string");

	let mut public = [0_u8; 32];
	let mut private = [0_u8; 64];

	let account = Account::new(seed);
	public.copy_from_slice(account.keys.pk.as_ref());
	private.copy_from_slice(account.keys.sk.as_ref());

	assert_eq!(public, account_id);
	assert_eq!(account.ss58(), "5G4oKgivg3aY1bkai9sFdPPV8LVL5pQi5gYgmic6qbL4y5Wm")
}

#[test]
fn test_to_ss58() {
	let mut seed:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"ad282c9eda80640f588f812081d98b9a2333435f76ba4ad6258e9c6f4a488363",
		&mut seed as &mut [u8])
		.expect("Cannot decode hex string");

	let account = Account::new(seed);
	let ss58 = account.ss58();
	assert_eq!(ss58, "5He7SpmVsdhoEKC5uwvPDngoCXECCeh8VrxoxnQTh1mgPiZa")
}