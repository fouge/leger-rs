mod tests;

pub trait Compact {
	fn scale_compact(&self, payload: &mut [u8]) -> usize;
}

impl Compact for u32 {
	fn scale_compact(&self, payload: &mut [u8]) -> usize {
		if *self < 64 {
			let casted = ((*self << 2) + 0) as u8;
			let i = casted.to_le_bytes();

			i.iter().zip(payload.iter_mut())
				.for_each(|(f, t)| *t = *f);
			i.len()
		} else if *self < (2_u32.pow(14) - 1) {
			let casted = ((*self << 2) + 1) as u16;
			let i = casted.to_le_bytes();

			i.iter().zip(payload.iter_mut())
				.for_each(|(f, t)| *t = *f);
			i.len()
		} else if *self < (2_u32.pow(30) - 1) {
			let i = ((*self << 2) + 2).to_le_bytes();

			i.iter().zip(payload.iter_mut())
				.for_each(|(f, t)| *t = *f);
			i.len()
		} else {
			let casted = *self as u128;
			return casted.scale_compact(payload)
		}
	}
}

impl Compact for u128 {
	fn scale_compact(&self, payload: &mut [u8]) -> usize {
		// check if goes into a u32
		if *self < (2_u32.pow(30) - 1) as u128 {
			let casted = *self as u32;
			return casted.scale_compact(payload)
		} else {
			let ba = self.to_le_bytes();
			let suffix_size = ba
				.iter().rev()
				.take_while(|&&x| x == 0)
				.count();

			let compact_size = ba.len()-suffix_size;

			payload[0] = 3 + ((compact_size - 4) << 2) as u8;
			ba[0..compact_size].iter().zip(payload[1..].iter_mut())
				.for_each(|(f, t)| *t = *f);

			return compact_size+1
		}
	}
}
