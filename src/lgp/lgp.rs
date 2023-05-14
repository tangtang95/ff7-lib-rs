use anyhow::{anyhow, Result};
use std::io::{Read, Seek, SeekFrom};

use super::{
    lgp_data::{LgpLookupTableEntry, LgpTocEntry},
    parser::lgp_file_header,
    LGP_FILE_HEADER_SIZE, LGP_LOOKUP_TABLE_SIZE,
};

pub struct Lgp<T: Read + Seek> {
    pub(super) reader: T,
    pub(super) toc: Vec<LgpTocEntry>,
    pub(super) lookup_table: [[LgpLookupTableEntry; LGP_LOOKUP_TABLE_SIZE]; LGP_LOOKUP_TABLE_SIZE],
    pub(super) byte_size: usize,
}

impl<T: Read + Seek> Lgp<T> {
    pub fn get_file(&mut self, filename: &str) -> Result<Vec<u8>> {
        let toc_entry = self
            .lookup(filename)
            .map(|table_entry| {
                self.toc
                    .iter()
                    .skip(table_entry.toc_offset - 1)
                    .take(table_entry.file_count)
                    .find(|toc_entry| toc_entry.filename.eq(filename))
            })
            .ok_or(anyhow!("Could not lookup filename in Lookup Table!"))?
            .ok_or(anyhow!("Could not find TOC entry matching filename!"))?;

        self.reader.seek(SeekFrom::Start(toc_entry.offset as u64))?;
        let mut buffer = [0u8; LGP_FILE_HEADER_SIZE];
        self.reader.read_exact(&mut buffer)?;

        let (_, lgp_file_header) = lgp_file_header(&buffer).map_err(|e| e.to_owned())?;

        let mut file_bytes = Vec::with_capacity(lgp_file_header.byte_size as usize);
        self.reader
            .by_ref()
            .take(lgp_file_header.byte_size as u64)
            .read_to_end(&mut file_bytes)?;
        Ok(file_bytes)
    }

    fn lookup(&self, filename: &str) -> Option<&LgpLookupTableEntry> {
        let mut filename_bytes = filename.bytes();
        let first_index = filename_bytes.next()? - b'a';
        let second_index = filename_bytes.next()? - b'a' + 1;

        self.lookup_table
            .get(first_index as usize)?
            .get(second_index as usize)
    }
}
