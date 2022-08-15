use crate::{
    app::{open_file, read_contents},
    args::Args,
    config::Config,
    shell::msg_warn,
    source::{Source, SourceAttributes, SourcePermissions, SourceProvider, SourceState},
    source_access::{Range, SourceAccess},
};

/// Copies data from source into a single contiguous buffer.
///
/// Holds all the accessible data in there.
pub struct SingleBufferAccessor(Vec<u8>);

impl SingleBufferAccessor {
    /// Create from an existing buffer
    pub(crate) fn from_vec(data: Vec<u8>) -> SingleBufferAccessor {
        Self(data)
    }
}

impl SourceAccess for SingleBufferAccessor {
    fn get_range(&mut self, range: Range) -> Option<&[u8]> {
        self.0.get(range)
    }
    fn get_range_mut(&mut self, range: Range) -> Option<&mut [u8]> {
        self.0.get_mut(range)
    }
    fn slice_range(&mut self, range: Range) -> &[u8] {
        &self.0[range]
    }
    fn slice_range_inclusive(&mut self, range: crate::source_access::RangeInclusive) -> &[u8] {
        &self.0[range]
    }
    fn slice_range_inclusive_mut(
        &mut self,
        range: crate::source_access::RangeInclusive,
    ) -> &mut [u8] {
        &mut self.0[range]
    }

    fn slice_range_from_upper_bound(
        &mut self,
        range: crate::source_access::RangeFrom,
        bound: usize,
    ) -> &[u8] {
        // Although we could slice more, we won't in order to have consistent debuggable behavior
        // with other implementors of SourceAccess.
        &self.0[range][..bound]
    }

    fn index_byte(&mut self, idx: usize) -> u8 {
        self.0[idx]
    }

    fn index_byte_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }

    fn source_len(&self) -> usize {
        self.0.len()
    }

    fn make_empty_and_free(&mut self) {
        self.0 = Vec::new();
    }

    fn downcast_to_single_buffer_vec(&mut self) -> Option<&mut Vec<u8>> {
        Some(&mut self.0)
    }

    fn open_file_from_args(
        &mut self,
        args: &mut Args,
        cfg: &mut Config,
        source: &mut Option<Source>,
    ) -> bool {
        if let Some(file_arg) = &args.file {
            if file_arg.as_os_str() == "-" {
                *source = Some(Source {
                    provider: SourceProvider::Stdin(std::io::stdin()),
                    attr: SourceAttributes {
                        seekable: false,
                        stream: true,
                        permissions: SourcePermissions {
                            read: true,
                            write: false,
                        },
                    },
                    state: SourceState::default(),
                });
                true
            } else {
                let result: Result<(), anyhow::Error> = try {
                    let mut file = open_file(file_arg, args.read_only)?;
                    self.0.clear();
                    if let Some(path) = &mut args.file {
                        match path.canonicalize() {
                            Ok(canon) => *path = canon,
                            Err(e) => msg_warn(&format!(
                                "Failed to canonicalize path {}: {}\n\
                                 Recent use list might not be able to load it back.",
                                path.display(),
                                e
                            )),
                        }
                    }
                    cfg.recent.use_(args.clone());
                    if !args.stream {
                        self.0 = read_contents(&*args, &mut file)?;
                    }
                    *source = Some(Source {
                        provider: SourceProvider::File(file),
                        attr: SourceAttributes {
                            seekable: true,
                            stream: args.stream,
                            permissions: SourcePermissions {
                                read: true,
                                write: !args.read_only,
                            },
                        },
                        state: SourceState::default(),
                    });
                };
                match result {
                    Ok(()) => true,
                    Err(e) => {
                        msg_warn(&format!("Failed to open file: {}", e));
                        false
                    }
                }
            }
        } else {
            false
        }
    }

    type Iter<'a> = impl Iterator<Item = u8> + 'a where Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().cloned()
    }

    type FindIter<'h, 'n> = impl Iterator<Item = usize> where Self: 'h;

    fn find_iter<'h, 'n>(&'h self, needle: &'n [u8]) -> Self::FindIter<'h, 'n> {
        memchr::memmem::find_iter(&self.0, needle)
    }

    fn get_range_from_upper_bound(
        &mut self,
        range: crate::source_access::RangeFrom,
        bound: usize,
    ) -> Option<&[u8]> {
        self.0.get(range).and_then(|r| r.get(..bound))
    }
}
