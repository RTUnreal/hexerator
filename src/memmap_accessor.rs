use std::fs::File;

use memmap2::{Mmap, MmapOptions};

use crate::source_access::SourceAccess;

pub struct MemmapAccessor {
    pub mmap: memmap2::Mmap,
}

impl SourceAccess for MemmapAccessor {
    fn get_range(&mut self, range: crate::source_access::Range) -> Option<&[u8]> {
        self.mmap.get(range)
    }

    fn slice_range(&mut self, range: crate::source_access::Range) -> &[u8] {
        &self.mmap[range]
    }

    fn get_range_mut(&mut self, range: crate::source_access::Range) -> Option<&mut [u8]> {
        todo!()
    }

    fn slice_range_inclusive(&mut self, range: crate::source_access::RangeInclusive) -> &[u8] {
        &self.mmap[range]
    }

    fn slice_range_inclusive_mut(
        &mut self,
        range: crate::source_access::RangeInclusive,
    ) -> &mut [u8] {
        todo!()
    }

    fn slice_range_from_upper_bound(
        &mut self,
        range: crate::source_access::RangeFrom,
        bound: usize,
    ) -> &[u8] {
        &self.mmap[range]
    }

    fn get_range_from_upper_bound(
        &mut self,
        range: crate::source_access::RangeFrom,
        bound: usize,
    ) -> Option<&[u8]> {
        self.mmap.get(range)
    }

    fn index_byte(&mut self, idx: usize) -> u8 {
        self.mmap[idx]
    }

    fn index_byte_mut(&mut self, idx: usize) -> &mut u8 {
        todo!()
    }

    fn source_len(&self) -> usize {
        self.mmap.len()
    }

    fn make_empty_and_free(&mut self) {
        todo!()
    }

    fn downcast_to_single_buffer_vec(&mut self) -> Option<&mut Vec<u8>> {
        None
    }

    fn open_file_from_args(
        &mut self,
        args: &mut crate::args::Args,
        cfg: &mut crate::config::Config,
        source: &mut Option<crate::source::Source>,
    ) -> bool {
        self.mmap = unsafe {
            MmapOptions::new()
                .len(20_000_000_000)
                .map(&File::open(args.file.clone().unwrap()).unwrap())
                .unwrap()
        };
        true
    }

    type Iter<'a> = impl Iterator<Item=u8>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.mmap.iter().cloned()
    }

    type FindIter<'h, 'n> = impl Iterator<Item=usize>
    where
        Self: 'h;

    fn find_iter<'h, 'n>(&'h self, needle: &'n [u8]) -> Self::FindIter<'h, 'n> {
        memchr::memmem::find_iter(&self.mmap, needle)
    }
}
