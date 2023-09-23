use anyhow::{anyhow, Result};
use std::{
    io::{Read, Seek, SeekFrom},
    slice::Iter,
};

use super::{
    lgp_data::{LgpFile, LgpLookupTableEntry, LgpTocEntry},
    parser::lgp_file_header,
    LGP_FILE_HEADER_SIZE, LGP_LOOKUP_TABLE_SIZE,
};

fn lgp_read_file_bytes<T: Read + Seek>(reader: &mut T, offset: u64) -> Result<Vec<u8>> {
    reader.seek(SeekFrom::Start(offset as u64))?;
    let mut buffer = [0u8; LGP_FILE_HEADER_SIZE];
    reader.read_exact(&mut buffer)?;

    let (_, lgp_file_header) = lgp_file_header(&buffer).map_err(|e| e.to_owned())?;

    let mut file_bytes = Vec::with_capacity(lgp_file_header.byte_size as usize);
    reader
        .by_ref()
        .take(lgp_file_header.byte_size as u64)
        .read_to_end(&mut file_bytes)?;
    Ok(file_bytes)
}

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

        lgp_read_file_bytes(&mut self.reader, toc_entry.offset as u64)
    }

    pub fn files_metadata_iter(&self) -> Iter<LgpTocEntry> {
        self.toc.iter()
    }

    pub fn num_files(&self) -> usize {
        self.toc.len()
    }

    pub fn num_bytes(&self) -> usize {
        self.byte_size
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

impl<T: Read + Seek> IntoIterator for Lgp<T> {
    type Item = LgpFile;
    type IntoIter = LgpIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            reader: self.reader,
            toc: self.toc,
            toc_idx: 0,
        }
    }
}

pub struct LgpIterator<T: Read + Seek> {
    pub(super) reader: T,
    pub(super) toc: Vec<LgpTocEntry>,
    pub(super) toc_idx: usize,
}

impl<T: Read + Seek> Iterator for LgpIterator<T> {
    type Item = LgpFile;

    fn next(&mut self) -> Option<Self::Item> {
        let toc_entry = self.toc.get(self.toc_idx)?;
        self.toc_idx += 1;
        let file_bytes = lgp_read_file_bytes(&mut self.reader, toc_entry.offset as u64).ok()?;
        Some(LgpFile {
            filename: toc_entry.filename.clone(),
            bytes: file_bytes,
        })
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufReader};
    use crate::lgp::parser::parse_lgp;
    use super::*;

    #[test]
    fn test() {
        let file = File::open("data/moviecam.lgp").unwrap();
        let buf_reader = BufReader::new(file);
        let mut lgp = parse_lgp(buf_reader).unwrap();

        for lgp_file in lgp.into_iter() {
            println!("{:?}", lgp_file.filename);
        }
    }
}
