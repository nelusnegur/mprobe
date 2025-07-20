use std::io::Error;
use std::io::Read;

use flate2::read::GzDecoder;
use tar::Archive;
use tar::Entries;
use tar::EntryType;

use crate::read::ReadItem;

pub(super) struct ReadArchive<R: Read> {
    archive: Archive<GzDecoder<R>>,
}

impl<R: Read> ReadArchive<R> {
    pub fn new(reader: R) -> Self {
        let dec = GzDecoder::new(reader);
        let archive = Archive::new(dec);

        Self { archive }
    }

    pub fn iter_mut(&mut self) -> Result<impl Iterator, Error> {
        let entries = self.archive.entries()?;
        Ok(ArchiveEntries::new(entries))
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(super) struct ArchiveEntries<'a, R: 'a + Read> {
    entries: Entries<'a, R>,
}

impl<'a, R: 'a + Read> ArchiveEntries<'a, R> {
    pub fn new(entries: Entries<'a, R>) -> Self {
        Self { entries }
    }
}

impl<'a, R: 'a + Read> Iterator for ArchiveEntries<'a, R> {
    type Item = Result<ReadItem<tar::Entry<'a, R>>, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_entry = self.entries.next()?;

            match next_entry {
                Ok(entry) if entry.header().entry_type() != EntryType::Regular => continue,
                Ok(entry) => {
                    let path = match entry.header().path() {
                        Ok(path) => path.into_owned(),
                        Err(error) => return Some(Err(error)),
                    };

                    return Some(ReadItem::new(&path, entry));
                }
                Err(error) => return Some(Err(error)),
            }
        }
    }
}
