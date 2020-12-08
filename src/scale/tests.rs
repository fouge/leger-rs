use crate::scale::Compact;

#[test]
fn test_scale_compact(){
	let mut payload = [0_u8; 16];
	let mut number_u128 = 123456789_u128;
	let mut count = number_u128.scale_compact(&mut payload);

	assert_eq!(count, 4);

	number_u128 = 2147483648_u128;
	count = number_u128.scale_compact(&mut payload);
	assert_eq!(count, 5);
	assert_eq!(payload[..count], [3, 0, 0, 0, 128]);

	let mut number_u32 = 123456_u32;
	count = number_u32.scale_compact(&mut payload);
	assert_eq!(count, 4);

	number_u32 = 1_u32;
	count = number_u32.scale_compact(&mut payload);
	assert_eq!(count, 1);

	number_u32 = 1536_u32;
	count = number_u32.scale_compact(&mut payload);
	assert_eq!(count, 2);

	number_u32 = 16384_u32;
	count = number_u32.scale_compact(&mut payload);
	assert_eq!(count, 4);
}