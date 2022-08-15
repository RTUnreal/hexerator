use crate::{args::Args, config::Config, source::Source};

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
