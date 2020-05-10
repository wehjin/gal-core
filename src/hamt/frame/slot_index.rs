use std::ops::Range;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SlotIndex {
	pub n: usize
}

impl SlotIndex {
	pub fn as_mask(&self) -> u32 { MASK_TABLE[self.n] }
	pub fn at(n: usize) -> Self {
		debug_assert!(n < 32);
		SlotIndex { n }
	}
	pub const RANGE: Range<usize> = (0..32);
}

const MASK_TABLE: &[u32; 32] = &[
	0x00000001, 0x00000002, 0x00000004, 0x00000008,
	0x00000010, 0x00000020, 0x00000040, 0x00000080,
	0x00000100, 0x00000200, 0x00000400, 0x00000800,
	0x00001000, 0x00002000, 0x00004000, 0x00008000,
	0x00010000, 0x00020000, 0x00040000, 0x00080000,
	0x00100000, 0x00200000, 0x00400000, 0x00800000,
	0x01000000, 0x02000000, 0x04000000, 0x08000000,
	0x10000000, 0x20000000, 0x40000000, 0x80000000,
];
