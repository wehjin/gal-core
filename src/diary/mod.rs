use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

pub use self::diary::Diary;
pub use self::reader::Reader;
pub use self::writer::Writer;

mod writer;
mod reader;
mod diary;

#[cfg(test)]
mod tests {
	use crate::{Point, Say, Sayer, ObjectId, Target};
	use crate::diary::{Diary, SayPos};

	#[test]
	fn main() {
		let start_say = Say { sayer: Sayer::Unit, object: ObjectId::Unit, point: Point::Unit, target: Some(Target::Number(3)) };
		let (path, pos) = {
			let diary = Diary::temp().unwrap();
			let mut writer = diary.writer().unwrap();
			let pos = writer.write_say(&start_say).unwrap();
			assert_eq!(pos, SayPos { sayer: 0.into(), object: 1.into(), point: 2.into(), target: 3.into(), end: (4 + 8).into() });
			diary.commit(writer.end_size());
			let mut commit_reader = diary.reader().unwrap();
			let commit_say = commit_reader.read_say(pos).unwrap();
			assert_eq!(commit_say, start_say);
			(diary.file_path.to_owned(), pos)
		};
		let reload_diary = Diary::load(&path).unwrap();
		let mut reload_reader = reload_diary.reader().unwrap();
		let reload_say = reload_reader.read_say(pos).unwrap();
		assert_eq!(reload_say, start_say);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Pos { start: usize }

impl Pos {
	pub fn at(start: usize) -> Self { Pos { start } }
	pub fn u32(&self) -> u32 { self.start as u32 }
}

impl fmt::Display for Pos {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(&format!("{}", self.start))
	}
}

impl From<usize> for Pos {
	fn from(n: usize) -> Self { Pos { start: n } }
}

impl From<Pos> for usize {
	fn from(pos: Pos) -> Self { pos.start as Self }
}

impl From<Pos> for u64 {
	fn from(pos: Pos) -> Self { pos.start as Self }
}

impl From<Pos> for u32 {
	fn from(pos: Pos) -> Self { pos.start as Self }
}

impl Add<Pos> for Pos {
	type Output = Pos;
	fn add(self, rhs: Pos) -> Self::Output {
		Pos { start: self.start + rhs.start }
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SayPos {
	pub sayer: Pos,
	pub object: Pos,
	pub point: Pos,
	pub target: Pos,
	pub end: Pos,
}

