use std::{io, thread};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

pub use write_scope::WriteScope;

use crate::{Chamber, diary, hamt, Say, Speech};
use crate::bytes::{ReadBytes, WriteBytes};
use crate::diary::Diary;
use crate::hamt::{Hamt, ProdAB, Root, ROOT_LEN};
use crate::util::io_error;

mod write_scope;

#[derive(Debug, Clone)]
pub struct Echo {
	tx: SyncSender<Action>,
}

enum Action {
	Speech(Speech, Sender<io::Result<Chamber>>),
	Latest(Sender<Chamber>),
}

impl Echo {
	/// Connects to an Echo.
	pub fn connect(name: &str, folder: impl AsRef<Path>) -> Self {
		let folder = folder.as_ref();
		let mut folder_path = folder.to_path_buf();
		folder_path.push(name);
		std::fs::create_dir_all(&folder_path).unwrap();
		let (tx, rx) = sync_channel::<Action>(64);
		thread::spawn(move || {
			let mut echo = InnerEcho::new(folder_path);
			for action in rx {
				match action {
					Action::Speech(speech, tx) => {
						let new_chamber = echo.write_speech(speech);
						tx.send(new_chamber).unwrap();
					}
					Action::Latest(tx) => {
						let chamber = echo.chamber().unwrap();
						tx.send(chamber).unwrap();
					}
				}
			}
		});
		Echo { tx }
	}

	/// Opens a scope for writing facts to the database and provides it to the
	/// given function.
	pub fn write<R>(&self, f: impl Fn(&mut WriteScope) -> R) -> io::Result<R> {
		let mut write = WriteScope { says: Vec::new() };
		let result = f(&mut write);
		self.write_speech(Speech { says: write.says })?;
		Ok(result)
	}

	fn write_speech(&self, speech: Speech) -> io::Result<Chamber> {
		let (tx, rx) = channel::<io::Result<Chamber>>();
		let action = Action::Speech(speech, tx);
		self.tx.send(action).unwrap();
		rx.recv().map_err(io_error)?
	}

	/// Constructs a chamber for reading facts from the database.
	pub fn chamber(&self) -> io::Result<Chamber> {
		let (tx, rx) = channel::<Chamber>();
		let action = Action::Latest(tx);
		self.tx.send(action).unwrap();
		rx.recv().map_err(io_error)
	}
}

struct InnerEcho {
	diary: Diary,
	diary_writer: diary::Writer,
	object_rings: Hamt,
	ring_objects: Hamt,
	roots_log: RootsLog,
}

impl InnerEcho {
	fn write_speech(&mut self, speech: Speech) -> io::Result<Chamber> {
		for say in speech.says.into_iter() {
			let mut diary_reader = self.diary_writer.reader()?;
			self.write_object_rings(&say, &mut diary_reader)?;
			self.write_ring_objects(&say, &mut diary_reader)?;
		}
		self.diary.commit(self.diary_writer.end_size());
		self.roots_log.write_roots(self.object_rings.root, self.ring_objects.root)?;
		self.chamber()
	}

	fn write_ring_objects(&mut self, say: &Say, diary_reader: &mut diary::Reader) -> io::Result<()> {
		let object_arrows_root = match self.ring_objects.reader()?.read_value(&say.ring, diary_reader)? {
			None => Root::ZERO,
			Some(root) => root
		};
		let mut object_arrows = Hamt::new(object_arrows_root);
		let arrow = match &say.arrow {
			None => unimplemented!(),
			Some(it) => it.clone(),
		};
		let object_arrow = ProdAB { a: say.object.to_owned(), b: arrow };
		object_arrows.write_value(&say.object, &object_arrow, &mut self.diary_writer)?;
		self.ring_objects.write_value(&say.ring, &object_arrows.root, &mut self.diary_writer)
	}

	fn write_object_rings(&mut self, say: &Say, diary_reader: &mut diary::Reader) -> io::Result<()> {
		let ring_arrows_root = match self.object_rings.reader()?.read_value(&say.object, diary_reader)? {
			None => Root::ZERO,
			Some(it) => it,
		};
		let mut ring_arrows = Hamt::new(ring_arrows_root);
		let arrow = match &say.arrow {
			None => unimplemented!(),
			Some(it) => it.clone(),
		};
		ring_arrows.write_value(&say.ring, &arrow, &mut self.diary_writer)?;
		self.object_rings.write_value(&say.object, &ring_arrows.root, &mut self.diary_writer)
	}

	fn chamber(&self) -> io::Result<Chamber> {
		let chamber = Chamber {
			ring_objects_reader: self.ring_objects.reader()?,
			object_rings_reader: self.object_rings.reader()?,
			diary_reader: self.diary.reader()?,
		};
		Ok(chamber)
	}

	fn new(folder_path: PathBuf) -> Self {
		let diary = Diary::load(&file_path("diary.dat", &folder_path)).unwrap();
		let diary_writer = diary.writer().unwrap();
		let roots_log = RootsLog::new(&folder_path).unwrap();
		let (object_rings_root, ring_objects_root) = roots_log.roots;
		let object_rings = Hamt::new(object_rings_root);
		let ring_objects = Hamt::new(ring_objects_root);
		InnerEcho { diary, diary_writer, object_rings, ring_objects, roots_log }
	}
}

struct RootsLog {
	appender: File,
	roots: (Root, Root),
}

impl RootsLog {
	pub fn write_roots(&mut self, a: Root, b: Root) -> io::Result<()> {
		let pos = self.appender.seek(SeekFrom::Current(0))?;
		let result = a.write_bytes(&mut self.appender)
			.and_then(|len| {
				assert_eq!(len, ROOT_LEN);
				b.write_bytes(&mut self.appender)
			})
			.map(|len| {
				assert_eq!(len, ROOT_LEN);
				()
			});
		if result.is_err() {
			self.appender.set_len(pos).unwrap();
			self.appender.seek(SeekFrom::Start(pos)).unwrap();
		}
		result
	}
	pub fn new(folder_path: &Path) -> io::Result<Self> {
		let file_path = file_path("roots.dat", folder_path);
		let appender = OpenOptions::new().create(true).append(true).open(&file_path)?;
		let roots = {
			let file_len = std::fs::metadata(&file_path)?.len();
			if file_len == 0 {
				(Root::ZERO, Root::ZERO)
			} else {
				let mut reader = OpenOptions::new().read(true).open(&file_path)?;
				reader.seek(SeekFrom::End(-2 * hamt::ROOT_LEN as i64))?;
				let a_root = Root::read_bytes(&mut reader)?;
				let b_root = Root::read_bytes(&mut reader)?;
				(a_root, b_root)
			}
		};
		Ok(RootsLog { appender, roots })
	}
}

fn file_path(file_name: &str, folder_path: &Path) -> PathBuf {
	let mut path = folder_path.to_path_buf();
	path.push(file_name);
	path
}
