use std::io::{Read, Write};
use std::io;

use crate::hamt::util;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Slot {
	Empty,
	Value { key: u32, value: u32 },
	Ref { pos: u32, mask: u32 },
}

impl Slot {
	pub fn read(source: &mut impl Read) -> io::Result<Slot> {
		let (flag, a, b) = read_data(source)?;
		let slot = if flag {
			Slot::Value { key: a, value: b }
		} else {
			Slot::Ref { pos: a, mask: b }
		};
		Ok(slot)
	}

	pub fn write(&self, dest: &mut impl Write) -> io::Result<usize> {
		match self {
			Slot::Empty => Ok(0),
			Slot::Value { key, value } => write_data(true, *key, *value, dest),
			Slot::Ref { pos, mask } => write_data(false, *pos, *mask, dest),
		}
	}
}

fn read_data(source: &mut impl Read) -> io::Result<(bool, u32, u32)> {
	let mut buf = [0u8; 8];
	source.read_exact(&mut buf)?;
	let flagged = (buf[0] & 0x80) == 0x80;
	buf[0] &= 0x7f;
	let (a, b) = util::u32x2_of_buf(&buf);
	Ok((flagged, a, b))
}

fn write_data(flag: bool, a: u32, b: u32, dest: &mut impl Write) -> io::Result<usize> {
	assert_eq!((a & 0x80), 0);
	let mut buf = [0u8; 4];
	util::big_end_first_4(a, &mut buf);
	if flag {
		buf[0] |= 0x80;
	}
	dest.write(&buf)?;
	util::big_end_first_4(b, &mut buf);
	dest.write(&buf)?;
	Ok(8)
}


