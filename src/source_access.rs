use crate::{
    args::Args, config::Config, memmap_accessor::MemmapAccessor,
    single_buffer_accessor::SingleBufferAccessor, source::Source,
};

pub type Range = std::ops::Range<usize>;
pub type RangeInclusive = std::ops::RangeInclusive<usize>;
pub type RangeFrom = std::ops::RangeFrom<usize>;

pub trait SourceAccess {
    /// Returns an immutable slice based on a range, Option-returning version.
    fn get_range(&mut self, range: Range) -> Option<&[u8]>;
    /// Returns an immutable slice based on a range
    fn slice_range(&mut self, range: Range) -> &[u8];
    /// Returns an optional mutable slice based on a range
    fn get_range_mut(&mut self, range: Range) -> Option<&mut [u8]>;
    /// Returns an immutable slice based on an inclusive range
    fn slice_range_inclusive(&mut self, range: RangeInclusive) -> &[u8];
    /// Returns a mutable slice based on an inclusive range
    fn slice_range_inclusive_mut(&mut self, range: RangeInclusive) -> &mut [u8];
    /// Returns an immutable slice based on a range-from, with an upper bound on how much to take.
    ///
    /// Not all accessors can just slice their entire source without allocating too much, so
    /// there must be a reasonable upper bound for how much to read.
    fn slice_range_from_upper_bound(&mut self, range: RangeFrom, bound: usize) -> &[u8];
    /// Option returning variant of [`slice_range_from_upper_bound`].
    fn get_range_from_upper_bound(&mut self, range: RangeFrom, bound: usize) -> Option<&[u8]>;
    /// Index a single byte by value
    fn index_byte(&mut self, idx: usize) -> u8;
    /// Index a single byte by mutable ref
    fn index_byte_mut(&mut self, idx: usize) -> &mut u8;
    /// Returns the full length of the source.
    ///
    /// Might be different than the length of whatever internal buffers
    /// we're using
    fn source_len(&self) -> usize;
    /// Make empty and free as much resources as we can.
    ///
    /// Usually called when closing a file, to free up previously used
    /// resources.
    fn make_empty_and_free(&mut self);
    /// Hack. Used to not break existing functionality that assumes single buffer accessor.
    ///
    /// TODO: Remove eventually.
    fn downcast_to_single_buffer_vec(&mut self) -> Option<&mut Vec<u8>>;
    /// Open a file from arguments, and get ready to serve access to it.
    ///
    /// Returns whether the file was loaded. It could be that it wasn't loaded
    /// for some reason.
    fn open_file_from_args(
        &mut self,
        args: &mut Args,
        cfg: &mut Config,
        source: &mut Option<Source>,
    ) -> bool;
    type Iter<'a>: Iterator<Item = u8>
    where
        Self: 'a;
    /// Iterate through all bytes of the source.
    fn iter(&self) -> Self::Iter<'_>;
    type FindIter<'h, 'n>: Iterator<Item = usize>
    where
        Self: 'h;
    /// Iterate through all bytes, return offsets that match the needle
    fn find_iter<'h, 'n>(&'h self, needle: &'n [u8]) -> Self::FindIter<'h, 'n>;
}

pub enum SourceAccessEnum {
    SingleBuffer(SingleBufferAccessor),
    Memmap(MemmapAccessor),
}

impl SourceAccess for SourceAccessEnum {
    fn get_range(&mut self, range: Range) -> Option<&[u8]> {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.get_range(range),
            SourceAccessEnum::Memmap(a) => a.get_range(range),
        }
    }

    fn slice_range(&mut self, range: Range) -> &[u8] {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.slice_range(range),
            SourceAccessEnum::Memmap(a) => a.slice_range(range),
        }
    }

    fn get_range_mut(&mut self, range: Range) -> Option<&mut [u8]> {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.get_range_mut(range),
            SourceAccessEnum::Memmap(a) => a.get_range_mut(range),
        }
    }

    fn slice_range_inclusive(&mut self, range: RangeInclusive) -> &[u8] {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.slice_range_inclusive(range),
            SourceAccessEnum::Memmap(a) => a.slice_range_inclusive(range),
        }
    }

    fn slice_range_inclusive_mut(&mut self, range: RangeInclusive) -> &mut [u8] {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.slice_range_inclusive_mut(range),
            SourceAccessEnum::Memmap(a) => a.slice_range_inclusive_mut(range),
        }
    }

    fn slice_range_from_upper_bound(&mut self, range: RangeFrom, bound: usize) -> &[u8] {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.slice_range_from_upper_bound(range, bound),
            SourceAccessEnum::Memmap(a) => a.slice_range_from_upper_bound(range, bound),
        }
    }

    fn get_range_from_upper_bound(&mut self, range: RangeFrom, bound: usize) -> Option<&[u8]> {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.get_range_from_upper_bound(range, bound),
            SourceAccessEnum::Memmap(a) => a.get_range_from_upper_bound(range, bound),
        }
    }

    fn index_byte(&mut self, idx: usize) -> u8 {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.index_byte(idx),
            SourceAccessEnum::Memmap(a) => a.index_byte(idx),
        }
    }

    fn index_byte_mut(&mut self, idx: usize) -> &mut u8 {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.index_byte_mut(idx),
            SourceAccessEnum::Memmap(a) => a.index_byte_mut(idx),
        }
    }

    fn source_len(&self) -> usize {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.source_len(),
            SourceAccessEnum::Memmap(a) => a.source_len(),
        }
    }

    fn make_empty_and_free(&mut self) {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.make_empty_and_free(),
            SourceAccessEnum::Memmap(a) => a.make_empty_and_free(),
        }
    }

    fn downcast_to_single_buffer_vec(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.downcast_to_single_buffer_vec(),
            SourceAccessEnum::Memmap(a) => a.downcast_to_single_buffer_vec(),
        }
    }

    fn open_file_from_args(
        &mut self,
        args: &mut Args,
        cfg: &mut Config,
        source: &mut Option<Source>,
    ) -> bool {
        match self {
            SourceAccessEnum::SingleBuffer(a) => a.open_file_from_args(args, cfg, source),
            SourceAccessEnum::Memmap(a) => a.open_file_from_args(args, cfg, source),
        }
    }

    type Iter<'a> = IterEnum<'a>;

    fn iter(&self) -> IterEnum {
        match self {
            SourceAccessEnum::SingleBuffer(s) => IterEnum::A(s.iter()),
            SourceAccessEnum::Memmap(m) => IterEnum::B(m.iter()),
        }
    }

    type FindIter<'h, 'n> = FindIterEnum<'h, 'n>;

    fn find_iter<'h, 'n>(&'h self, needle: &'n [u8]) -> FindIterEnum<'h, 'n> {
        match self {
            SourceAccessEnum::SingleBuffer(s) => FindIterEnum::A(s.find_iter(needle)),
            SourceAccessEnum::Memmap(m) => FindIterEnum::B(m.find_iter(needle)),
        }
    }
}

type IterA<'a> = impl Iterator<Item = u8>;
type IterB<'a> = impl Iterator<Item = u8>;
pub enum IterEnum<'a> {
    A(IterA<'a>),
    B(IterB<'a>),
}

impl<'a> Iterator for IterEnum<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterEnum::A(a) => a.next(),
            IterEnum::B(b) => b.next(),
        }
    }
}

type FindIterA<'h, 'n> = impl Iterator<Item = usize>;
type FindIterB<'h, 'n> = impl Iterator<Item = usize>;
pub enum FindIterEnum<'h, 'n> {
    A(FindIterA<'h, 'n>),
    B(FindIterB<'h, 'n>),
}

impl<'h, 'n> Iterator for FindIterEnum<'h, 'n> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FindIterEnum::A(a) => a.next(),
            FindIterEnum::B(b) => b.next(),
        }
    }
}
