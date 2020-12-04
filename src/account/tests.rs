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

	let account = Account::new(seed);

	let mut public = [0_u8; 32];
	public.copy_from_slice(account.keys.pk.as_ref());

	assert_eq!(public, account_id);

	let s = account.ss58();
	assert_eq!(s, "5He7SpmVsdhoEKC5uwvPDngoCXECCeh8VrxoxnQTh1mgPiZa")
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

#[test]
fn test_sign_payload() {
	let mut seed:[u8; 32] = [0_u8; 32];
	let mut payload:[u8; 111] = [0_u8; 111];
	let mut signature:[u8; 64] = [0_u8; 64];

	hex::decode_to_slice(
		"DA5CE9BB3618B9004F0D76C0FE97DA6C075AE60937FC7B3A8C01A16A655E9388",
		&mut seed as &mut [u8])
		.expect("Cannot decode hex string");

	// payload
	hex::decode_to_slice(
		"0500FF67A65330BC82ED395DF1CC6C818E01ED57DCF8A6CCC3D8785EA965CCD917F3A0280000000100000001000000948C3E4175EAD6E21506A948CD93FB27C4C0A0D4F59BD3EEFE03DE9EEA5E5918948C3E4175EAD6E21506A948CD93FB27C4C0A0D4F59BD3EEFE03DE9EEA5E5918",
		&mut payload as &mut [u8])
		.expect("Cannot decode hex string");

	// signature
	hex::decode_to_slice(
	"AB94EE445A4B3E5E9EF915A9C50F3A90D8E5F1F30DC7DDC942471B43F51B44FFF15B8131131ACCD7796835B614E0C7B4E0B07860AB3027CE507743A239B35B0C",
		&mut signature as &mut [u8])
		.expect("Cannot decode hex string");

	let account = Account::new(seed);
	account.sign_tx(&mut payload);

	assert_eq!(payload[0..64], signature)
}