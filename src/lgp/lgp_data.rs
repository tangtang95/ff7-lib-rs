#[derive(Debug)]
pub(super) struct LgpHeader {
    pub file_count: usize,
}

/// Lgp Table of Contents (ToC) Entry
#[derive(Debug)]
pub(super) struct LgpTocEntry {
    pub filename: String,
    pub offset: u32,
    pub code: u8,
    pub duplicate: Option<u16>,
}

/// LGP Lookup table entry for the Table of Contents (ToC)
#[derive(Debug, Default, Copy, Clone)]
pub(super) struct LgpLookupTableEntry {
    pub toc_offset: usize,
    pub file_count: usize,
}

pub(super) struct LgpFileHeader {
    pub filename: String,
    pub byte_size: u32,
}