pub mod transfer;

/// Implement the Call trait to add the use the encoded data as Call
pub trait Call {
	fn encode(&self, payload: &mut [u8]) -> usize;
}
