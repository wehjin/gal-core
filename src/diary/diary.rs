use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use crate::diary::{Reader, Writer};

pub struct Diary {
	pub file_path: PathBuf,
	pub file_size: usize,
}

impl Diary {
	pub fn reader(&self) -> io::Result<Reader> {
		let file = OpenOptions::new().read(true).open(&self.file_path)?;
		let file_size = self.file_size;
		Ok(Reader { file, file_size })
	}

	pub fn commit(&mut self, writer: &Writer) {
		self.file_size = writer.file_size;
	}

	pub fn writer(&self) -> io::Result<Writer> {
		let file = OpenOptions::new().append(true).create(true).open(&self.file_path)?;
		let file_size = self.file_size;
		file.set_len(file_size as u64)?;
		Ok(Writer { file, file_size })
	}

	pub fn len_in_bytes(&self) -> usize { self.file_size }

	pub fn temp() -> io::Result<Diary> {
		let mut path = std::env::temp_dir();
		path.push(&format!("diary{}", rand::random::<u32>()));
		Diary::load(&path)
	}

	pub fn load(file_path: &Path) -> io::Result<Diary> {
		let file_path = file_path.to_path_buf();
		let file_size = {
			let file = OpenOptions::new().write(true).create(true).open(&file_path)?;
			file.metadata()?.len() as usize
		};
		Ok(Diary { file_path, file_size })
	}
}