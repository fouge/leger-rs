pub mod transfer;

/// Any Call object can be included into an extrinsic
/// Implement this trait to add extrinsic calls implementation
pub trait Call {
	fn encode(&self, payload: &mut [u8]) -> usize;
}
