pub trait Chain {
	type Error: core::fmt::Debug;

	fn get_block_hash(&mut self, number: Option<[usize; 1]>) -> Result<[u8; 32], Self::Error>;
	fn get_genesis_block_hash(&mut self) -> Result<[u8; 32], Self::Error>;
	fn get_finalized_head(&mut self) -> Result<&str, Self::Error>;
}