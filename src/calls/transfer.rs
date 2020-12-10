use crate::calls::Call;
use crate::scale::Compact;

pub struct ExtrinsicTransferCall {
	module_idx: u8,
	call_idx: u8,
	is_address: u8,
	dest_account: [u8; 32],
	amount: u128 // SCALE encoded
}

impl ExtrinsicTransferCall {
	pub fn new(dest_account: &[u8; 32], amount: u128) -> ExtrinsicTransferCall {
		let mut e = ExtrinsicTransferCall {
			module_idx: 5, // balances
			call_idx: 0,
			is_address: 0xFF,
			dest_account: [0_u8; 32],
			amount
		};

		e.dest_account.clone_from_slice(dest_account);

		e
	}
}

impl Call for ExtrinsicTransferCall {
	fn encode(&self, payload: &mut [u8]) -> usize {
		payload[0] = self.module_idx;
		payload[1] = self.call_idx;

		// we support only account ID as u8
		//payload[2] = 0xff;
		payload[2..2+self.dest_account.len()].copy_from_slice(self.dest_account.as_ref());
		let mut idx = 2 + self.dest_account.len();

		idx += self.amount.scale_compact(&mut payload[idx..]);

		idx
	}
}