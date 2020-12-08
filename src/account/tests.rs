use crate::account::Account;
use hex;

/// Test key creation from secret seed.
/// Private and public keys taken from https://substrate.dev/docs/en/knowledgebase/integrate/subkey
#[test]
fn test_new_account() {
	let mut seed:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"554b6fc625fbea8f56eb56262d92ccb083fd6eaaf5ee9a966eaab4db2062f4d0",
		&mut seed as &mut [u8])
		.expect("Cannot decode hex string");
	let mut account_id:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"143fa4ecea108937a2324d36ee4cbce3c6f3a08b0499b276cd7adb7a7631a559",
		&mut account_id as &mut [u8])
		.expect("Cannot decode hex string");

	let account = Account::new(seed);

	let mut public = [0_u8; 32];
	public.copy_from_slice(account.keys.public.as_ref());

	assert_eq!(public, account_id);

	let s = account.ss58();
	assert_eq!(s, "5CXFinBHRrArHzmC6iYVHSSgY1wMQEdL2AiL6RmSEsFvWezd")
}

#[test]
fn test_alice_account() {
	// bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice
	let mut seed:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		&mut seed as &mut [u8])
		.expect("Cannot decode hex string");
	let mut account_id:[u8; 32] = [0_u8; 32];
	hex::decode_to_slice(
		"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
		&mut account_id as &mut [u8])
		.expect("Cannot decode hex string");

	let mut public = [0_u8; 32];
	let mut private = [0_u8; 64];

	let account = Account::new(seed);
	public.copy_from_slice(account.keys.public.to_bytes().as_ref());
	private.copy_from_slice(account.keys.secret.to_bytes().as_ref());

	assert_eq!(public, account_id);
	assert_eq!(account.ss58(), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
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
	let mut signature_to_find:[u8; 64] = [0_u8; 64];

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
		&mut signature_to_find as &mut [u8])
		.expect("Cannot decode hex string");

	let account = Account::new(seed);
	account.sign_tx(&mut payload, &mut signature);

	assert_eq!(signature, signature_to_find)
}