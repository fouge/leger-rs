use crate::account::Account;

pub trait Calls {
	type Error: core::fmt::Debug;

	fn balance_transfer(&mut self, source_account: &mut Account, dest_account: &[u8; 32], amount: u128)
		-> Result<&str, Self::Error>;
}