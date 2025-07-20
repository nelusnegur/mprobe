use std::fs;
use std::fs::File;
use std::fs::ReadDir;
use std::io::Error;

use crate::read::ReadItem;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub(super) struct ReadDirectory {
    dirs: Vec<ReadDir>,
}

impl ReadDirectory {
    pub fn new(root_dir: ReadDir) -> Self {
        let dirs = vec![root_dir];
        Self { dirs }
    }
}

impl Iterator for ReadDirectory {
    type Item = Result<ReadItem<File>, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let dir = self.dirs.last_mut()?;

            match dir.next() {
                Some(Ok(entry)) => match entry.file_type() {
                    Ok(file_type) => {
                        if file_type.is_dir() {
                            match fs::read_dir(entry.path()) {
                                Ok(next_dir) => self.dirs.push(next_dir),
                                Err(error) => return Some(Err(error)),
                            }
                        } else if file_type.is_file() {
                            // TODO: Process the .interim file last
                            // For now just skip it.
                            let path = entry.path();
                            if path.extension().is_none_or(|e| e == "interim") {
                                continue;
                            }

                            let read_item = match File::open(&path) {
                                Ok(file) => ReadItem::new(&path, file),
                                Err(error) => return Some(Err(error)),
                            };

                            return Some(read_item);
                        } else {
                            continue;
                        }
                    }
                    Err(error) => return Some(Err(error)),
                },
                Some(Err(error)) => return Some(Err(error)),
                None => {
                    self.dirs.pop();
                }
            }
        }
    }
}
