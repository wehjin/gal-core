use std::hash::Hash;

pub use object::*;
pub use point::*;
pub use target::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub says: Vec<Say>
}

pub trait Writable {
	fn to_says(&self) -> Vec<Say>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Say {
	pub sayer: Sayer,
	pub object: ObjName,
	pub point: Point,
	pub target: Option<Target>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	Unit,
	Named(String),
}

mod object;
mod point;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Target {
	Number(u64),
	Text(String),
}

impl Target {
	pub fn as_number(&self) -> u64 {
		match self {
			Target::Number(n) => *n,
			_ => panic!("Target is not a number")
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Target::Text(s) => s,
			_ => panic!("Target is not text")
		}
	}
}

mod target;

