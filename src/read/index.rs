//! `FileIndex` implementations

use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Result, Seek, SeekFrom};
use std::iter::{self, Extend};
use std::ops::Range;

use super::{Entry, Section};
use crate::Compression;

/// The storage used to store file indices.
///
/// If you open the phar only to view the stub, phar metadata, etc.,
/// use `index::NoIndex`.
///
/// To sequentially access all files in the archive,
/// use `index::Iterable` implementations.
/// If random file access is not required,
/// use `index::OffsetOnly`.
///
/// To access specific files only,
/// use `index::RandomAccess` implementations.
/// Prefer using `NameMap` if individual entry metadata is not required.
/// To also access their metadata,
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
}

/// A subfamily of file indices for iterable files.
///
/// The iteration order may not be the order in the phar archive,
/// and may not even be stable.
pub trait Iterable: FileIndex {
    /// Iterates over the files in this index.
    fn for_each_file<'t, R, F>(&self, read: R, f: F) -> Result<()>
    where
        R: Read + Seek + 't,
        F: FnMut(&[u8], &mut (dyn Read)) -> Result<()>,
    {
        self.for_each_file_fold(read, f, |_, ()| ()).map(|_| ())
    }

    /// Iterates over the files in this index and fold return values.
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

/// Indicates that the phar should not index phar files at all.
///
/// This should only be used if phar files are not going to be accessed,
/// or allocating `O(num_files)` memory is considered a security vulnerability.
#[derive(Debug, Default)]
pub struct NoIndex(());

impl FileIndex for NoIndex {
    fn scan_files() -> bool {
        false
    }

    fn feed_entry(&mut self, _: u64, _: Entry) -> Result<()> {
        unreachable!()
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
    entries: Vec<OffsetOnlyEntry>,
}

#[derive(Debug)]
struct OffsetOnlyEntry {
    name: Section,
    flags: u32,
    end_offset_from_co: u64,
}

impl FileIndex for OffsetOnly {
    fn feed_entry(&mut self, _: u64, entry: Entry) -> Result<()> {
        let prev = match self.entries.last() {
            Some(ooe) => ooe.end_offset_from_co,
            None => 0,
        };
        let size: u64 = entry.compressed_file_size.into();
        self.entries.push(OffsetOnlyEntry {
            name: entry.name,
            flags: entry.flags,
            end_offset_from_co: prev + size,
        });
        Ok(())
    }

    fn end_of_header(&mut self, offset: u64) {
        self.content_offset = offset;
    }
}

impl Iterable for OffsetOnly {
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

        for OffsetOnlyEntry {
            name,
            flags,
            end_offset_from_co,
        } in &self.entries
        {
            let name = name.as_memory(&mut read)?;
            let name = name.as_ref();
            let end_offset = *end_offset_from_co + self.content_offset;

            let _ = read.seek(SeekFrom::Start(start_offset))?;
            let mut decompressed =
                adapted_reader(*flags, (&mut read).take(end_offset - start_offset))?;
            let mapped = f(name, &mut decompressed)?;
            reduced = Some(fold(reduced, mapped));

            start_offset = end_offset;
        }

        Ok(reduced)
    }
}

/// Indexes files by name for random access.
#[derive(Debug, Default)]
pub struct NameMap<M> {
    map: M,
    last_offset: u64,
    content_offset: u64,
}

impl<M: Default + Extend<(Vec<u8>, (u32, Range<u64>))>> FileIndex for NameMap<M>
where
    for<'t> &'t M: IntoIterator<Item = (&'t Vec<u8>, &'t (u32, Range<u64>))>,
{
    fn requires_name() -> bool {
        true
    }

    fn end_of_header(&mut self, offset: u64) {
        self.content_offset = offset;
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
}

impl<M: Default + Extend<(Vec<u8>, (u32, Range<u64>))>> Iterable for NameMap<M>
where
    for<'t> &'t M: IntoIterator<Item = (&'t Vec<u8>, &'t (u32, Range<u64>))>,
{
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
            let _ = read.seek(SeekFrom::Start(*start + self.content_offset))?;
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
    content_offset: u64,
}

impl<M: Default + Extend<(Vec<u8>, (Entry, Range<u64>))>> FileIndex for MetadataMap<M>
where
    for<'t> &'t M: IntoIterator<Item = (&'t Vec<u8>, &'t (Entry, Range<u64>))>,
{
    fn requires_name() -> bool {
        true
    }

    fn requires_metadata() -> bool {
        true
    }

    fn end_of_header(&mut self, offset: u64) {
        self.content_offset = offset;
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
}

impl<M: Default + Extend<(Vec<u8>, (Entry, Range<u64>))>> Iterable for MetadataMap<M>
where
    for<'t> &'t M: IntoIterator<Item = (&'t Vec<u8>, &'t (Entry, Range<u64>))>,
{
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
            let _ = read.seek(SeekFrom::Start(*start + self.content_offset))?;
            let mut decompressed = adapted_reader(entry.flags, (&mut read).take(end - start))?;
            let mapped = f(name, &mut decompressed)?;
            reduced = Some(fold(reduced, mapped));
        }

        Ok(reduced)
    }
}

/// Indexes files by name with a HashMap, and stores file metadata.
pub type MetadataHashMap = MetadataMap<HashMap<Vec<u8>, (Entry, Range<u64>)>>;

/// Indexes files by name with a BTreeMap, and stores file metadata.
pub type MetadataBTreeMap = MetadataMap<BTreeMap<Vec<u8>, (Entry, Range<u64>)>>;

fn adapted_reader<'t>(flag: u32, read: impl Read + 't) -> Result<Box<(dyn Read + 't)>> {
    let compression = Compression::from_bit(flag);
    compression.make_read(read)
}
