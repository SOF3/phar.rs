use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Result, Seek, SeekFrom};
use std::iter::{self, Extend};
use std::ops::Range;

use cfg_if::cfg_if;

use super::{Entry, Section};

/// The storage used to store file indices.
///
/// If you open the phar only to view the stub, phar metadata, etc.,
/// use `NoIndex`.
/// You may also want to use `NoIndex`
/// if you are worried about the phar having too many files
/// and overloading the memory allocated for loading the phar.
///
/// If you are not worried about storing a large amount of file names in memory,
/// use `OffsetOnly`.
/// This allows sequential iteration of files.
///
/// To access specific files only,
/// prefer using `NameMap`.
/// To also access thei metadata,
/// use `MetadataMap`.
/// There are some type aliases for the respective HashMap/BTreeMap implementations.
pub trait FileIndex: Default {
    /// Whether file metadata should be scanned on loading.
    fn scan_files() -> bool {
        true
    }

    /// Whether `Entry` should force `name` to use `Section::Cached`.
    fn requires_name() -> bool {
        false
    }

    /// Whether `Entry` should force `metadata` to use `Section::Cached`.
    fn requires_metadata() -> bool {
        false
    }

    /// Adds an `Entry` to the index.
    fn feed_entry(&mut self, offset: u64, entry: Entry) -> Result<()>;

    /// Marks the end of header
    fn end_of_header(&mut self, _offset: u64) {}

    /// Iterates over the file in this index.
    fn for_each_file<'t, R, F>(&self, read: R, mut f: F) -> Result<()>
    where
        R: Read + Seek + 't,
        F: FnMut(&[u8], &mut (dyn Read)),
    {
        self.for_each_file_fold(
            read,
            |name, r| {
                f(name, r);
                Ok(())
            },
            |_, ()| (),
        )
        .map(|_| ())
    }

    /// Iterates over the file in this index and fold return values.
    fn for_each_file_fold<'t, R, F, G, T, U>(&self, read: R, f: F, fold: G) -> Result<Option<T>>
    where
        R: Read + Seek + 't,
        F: FnMut(&[u8], &mut (dyn Read)) -> Result<U>,
        G: FnMut(Option<T>, U) -> T;
}

/// A subfamily of file indices for random access of files by name.
pub trait RandomAccess: FileIndex {
    /// Returns the file contents range of the file of the required name.
    ///
    /// Returns `None` if there are no files with the specified name.
    fn read_file(&self, name: &[u8]) -> Option<Range<u64>>;
}

#[derive(Debug, Default)]
pub struct NoIndex {
    first_entry_offset: Option<u64>,
    num_files: u32,
    end_of_header: u64,
}

impl FileIndex for NoIndex {
    fn scan_files() -> bool {
        false
    }

    fn feed_entry(&mut self, offset: u64, _: Entry) -> Result<()> {
        self.first_entry_offset = Some(match self.first_entry_offset {
            Some(first) => cmp::min(first, offset),
            None => offset,
        });
        self.num_files += 1;
        Ok(())
    }

    fn end_of_header(&mut self, offset: u64) {
        self.end_of_header = offset;
    }

    fn for_each_file_fold<'t, R, F, G, T, U>(
        &self,
        mut read: R,
        mut f: F,
        mut fold: G,
    ) -> Result<Option<T>>
    where
        R: Read + Seek + 't,
        F: FnMut(&[u8], &mut (dyn Read)) -> Result<U>,
        G: FnMut(Option<T>, U) -> T,
    {
        let mut seek_manifest = match self.first_entry_offset {
            Some(start) => start,
            None => return Ok(None),
        };
        let mut seek_content = self.end_of_header;

        let mut reduced = None;

        for _ in 0..self.num_files {
            let _ = read.seek(SeekFrom::Start(seek_manifest))?;
            let entry = Entry::parse(&mut read, true, false)?;
            seek_manifest = read.seek(SeekFrom::Current(0))?;

            let file_name = entry.name.as_memory(&mut read)?;
            let file_name = file_name.as_ref();
            let _ = read.seek(SeekFrom::Start(seek_content))?;

            let take = (&mut read).take(entry.compressed_file_size.into());
            let mut decompressed = adapted_reader(entry.flags, take)?;
            let mapped = f(file_name, &mut decompressed)?;
            reduced = Some(fold(reduced, mapped));

            seek_content += u64::from(entry.compressed_file_size);
        }

        Ok(reduced)
    }
}

/// Stores files by position.
///
/// Uses only `O(nm)` memory,
/// where `n` is the number of files,
/// and `m` is either `1` or the length of filenames
/// depending on whether files are cached.
#[derive(Debug, Default)]
pub struct OffsetOnly {
    content_offset: u64,
    entries: Vec<(Section, u32, u64)>,
}

impl FileIndex for OffsetOnly {
    fn feed_entry(&mut self, _: u64, entry: Entry) -> Result<()> {
        let prev = match self.entries.last() {
            Some((_, _, last)) => *last,
            None => 0,
        };
        let size: u64 = entry.compressed_file_size.into();
        self.entries.push((entry.name, entry.flags, prev + size));
        Ok(())
    }

    fn end_of_header(&mut self, offset: u64) {
        self.content_offset = offset;
    }

    fn for_each_file_fold<'t, R, F, G, T, U>(
        &self,
        mut read: R,
        mut f: F,
        mut fold: G,
    ) -> Result<Option<T>>
    where
        R: Read + Seek + 't,
        F: FnMut(&[u8], &mut (dyn Read)) -> Result<U>,
        G: FnMut(Option<T>, U) -> T,
    {
        let mut start_offset = self.content_offset;
        let mut reduced = None;

        for (name, flags, end_offset) in &self.entries {
            let name = name.as_memory(&mut read)?;
            let name = name.as_ref();

            let _ = read.seek(SeekFrom::Start(start_offset))?;
            let mut decompressed =
                adapted_reader(*flags, (&mut read).take(*end_offset - start_offset))?;
            let mapped = f(name, &mut decompressed)?;
            reduced = Some(fold(reduced, mapped));

            start_offset = *end_offset;
        }

        Ok(reduced)
    }
}

/// Indexes files by name for random access.
#[derive(Debug, Default)]
pub struct NameMap<M> {
    map: M,
    last_offset: u64,
}

impl<M: Default + Extend<(Vec<u8>, (u32, Range<u64>))>> FileIndex for NameMap<M>
where
    for<'t> &'t M: Iterator<Item = (&'t Vec<u8>, &'t (u32, Range<u64>))>,
{
    fn requires_name() -> bool {
        true
    }

    fn feed_entry(&mut self, _: u64, entry: Entry) -> Result<()> {
        let len: u64 = entry.compressed_file_size.into();

        let name = match entry.name {
            Section::Cached(cache) => cache,
            _ => unreachable!("requires_name is set to true"),
        };
        let start = self.last_offset;
        let end = start + len;
        self.last_offset = end;
        self.map
            .extend(iter::once((name, (entry.flags, start..end))));
        Ok(())
    }

    fn for_each_file_fold<'t, R, F, G, T, U>(
        &self,
        mut read: R,
        mut f: F,
        mut fold: G,
    ) -> Result<Option<T>>
    where
        R: Read + Seek + 't,
        F: FnMut(&[u8], &mut (dyn Read)) -> Result<U>,
        G: FnMut(Option<T>, U) -> T,
    {
        let mut reduced = None;

        for (name, (flags, Range { start, end })) in &self.map {
            let _ = read.seek(SeekFrom::Start(*start))?;
            let mut decompressed = adapted_reader(*flags, (&mut read).take(end - start))?;
            let mapped = f(name, &mut decompressed)?;
            reduced = Some(fold(reduced, mapped));
        }

        Ok(reduced)
    }
}

/// Indexes files by name with a HashMap.
pub type NameHashMap = NameMap<HashMap<Vec<u8>, (u32, Range<u64>)>>;
/// Indexes files by name with a BTreeMap.
pub type NameBTreeMap = NameMap<BTreeMap<Vec<u8>, (u32, Range<u64>)>>;

/// Indexes files by name for random access, and stores file metadata.
#[derive(Debug, Default)]
pub struct MetadataMap<M> {
    pub(crate) map: M,
    last_offset: u64,
}

impl<M: Default + Extend<(Vec<u8>, (Entry, Range<u64>))>> FileIndex for MetadataMap<M>
where
    for<'t> &'t M: Iterator<Item = (&'t Vec<u8>, &'t (Entry, Range<u64>))>,
{
    fn requires_name() -> bool {
        true
    }

    fn requires_metadata() -> bool {
        true
    }

    fn feed_entry(&mut self, _: u64, entry: Entry) -> Result<()> {
        let name = match &entry.name {
            Section::Cached(cache) => cache,
            _ => unreachable!("requires_name is set to true"),
        };
        let start = self.last_offset;
        let len: u64 = entry.compressed_file_size.into();
        let end = start + len;
        self.last_offset = end;
        self.map
            .extend(iter::once((name.clone(), (entry, start..end))));
        Ok(())
    }

    fn for_each_file_fold<'t, R, F, G, T, U>(
        &self,
        mut read: R,
        mut f: F,
        mut fold: G,
    ) -> Result<Option<T>>
    where
        R: Read + Seek + 't,
        F: FnMut(&[u8], &mut (dyn Read)) -> Result<U>,
        G: FnMut(Option<T>, U) -> T,
    {
        let mut reduced = None;

        for (name, (entry, Range { start, end })) in &self.map {
            let _ = read.seek(SeekFrom::Start(*start))?;
            let mut decompressed = adapted_reader(entry.flags, (&mut read).take(end - start))?;
            let mapped = f(name, &mut decompressed)?;
            reduced = Some(fold(reduced, mapped));
        }

        Ok(reduced)
    }
}

/// Indexes files by name with a HashMap, and stores file metadata.
pub type MetadataHashMap = MetadataMap<HashMap<Vec<u8>, Range<u64>>>;

/// Indexes files by name with a BTreeMap, and stores file metadata.
pub type MetadataBTreeMap = MetadataMap<BTreeMap<Vec<u8>, Range<u64>>>;

#[allow(clippy::unnecessary_wraps)]
fn adapted_reader<'t>(flag: u32, r: impl Read + 't) -> Result<Box<(dyn Read + 't)>> {
    if (flag & 0x1000) > 0 {
        cfg_if! {
            if #[cfg(feature = "comp-zlib")] {
                Ok(Box::new(flate2::read::ZlibDecoder::new(r)))
            } else {
                Err(Error::new(ErrorKind::Other, "Compile the phar crate with comp-zlib feature to use zlib-compressed files"))
            }
        }
    } else if (flag & 0x2000) > 0 {
        cfg_if! {
            if #[cfg(feature = "comp-bzip")] {
                Ok(Box::new(bzip2::read::BzDecoder::new(r)))
            } else {
                Err(Error::new(ErrorKind::Other, "Compile the phar crate with comp-bzip feature to use bzip-compressed files"))
            }
        }
    } else {
        Ok(Box::new(r))
    }
}
