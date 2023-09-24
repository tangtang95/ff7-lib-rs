use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

use nom::{
    bytes::complete::{tag, take},
    combinator::map_res,
    multi::{count, fill},
    number::complete::{le_u16, le_u32, u8},
    sequence::Tuple,
    IResult,
};

use super::{
    lgp::Lgp,
    lgp_data::{LgpFileHeader, LgpHeader, LgpLookupTableEntry, LgpTocEntry},
    LGP_HEADER_SIZE, LGP_LOOKUP_TABLE_SIZE, LGP_PRODUCT_NAME_SIZE, LGP_TOC_FILENAME_SIZE,
    LGP_TOC_SIZE,
};

fn ascii_string(input: &[u8], length: usize) -> IResult<&[u8], String> {
    map_res(take(length), |filename| {
        std::str::from_utf8(filename).map(|text| text.trim_end_matches('\0').to_owned())
    })(input)
}

fn lgp_header(input: &[u8]) -> IResult<&[u8], LgpHeader> {
    let (input, _) = tag(b"\0\0SQUARESOFT")(input)?;
    let (input, file_count) = le_u32(input)?;
    Ok((
        input,
        LgpHeader {
            file_count: file_count as usize,
        },
    ))
}

fn lgp_toc_entry(input: &[u8]) -> IResult<&[u8], LgpTocEntry> {
    let (input, filename) = ascii_string(input, LGP_TOC_FILENAME_SIZE)?;
    let (input, (offset, code, duplicate)) = (le_u32, u8, le_u16).parse(input)?;
    Ok((
        input,
        LgpTocEntry {
            filename,
            offset,
            code,
            duplicate: if duplicate == 0 {
                None
            } else {
                Some(duplicate)
            },
        },
    ))
}

fn lgp_toc(input: &[u8], num_files: usize) -> IResult<&[u8], Vec<LgpTocEntry>> {
    count(lgp_toc_entry, num_files)(input)
}

fn lgp_lookup_table_entry(input: &[u8]) -> IResult<&[u8], LgpLookupTableEntry> {
    let (input, (toc_offset, file_count)) = (le_u16, le_u16).parse(input)?;
    Ok((
        input,
        LgpLookupTableEntry {
            toc_offset: toc_offset.into(),
            file_count: file_count.into(),
        },
    ))
}

fn lgp_lookup_table(
    mut input: &[u8],
) -> IResult<&[u8], [[LgpLookupTableEntry; LGP_LOOKUP_TABLE_SIZE]; LGP_LOOKUP_TABLE_SIZE]> {
    let mut lookup_table =
        [[LgpLookupTableEntry::default(); LGP_LOOKUP_TABLE_SIZE]; LGP_LOOKUP_TABLE_SIZE];
    for i in 0..LGP_LOOKUP_TABLE_SIZE {
        (input, _) = fill(lgp_lookup_table_entry, &mut lookup_table[i])(input)?;
    }
    Ok((input, lookup_table))
}

pub(super) fn lgp_file_header(input: &[u8]) -> IResult<&[u8], LgpFileHeader> {
    let (input, filename) = ascii_string(input, LGP_TOC_FILENAME_SIZE)?;
    let (input, byte_size) = le_u32(input)?;
    Ok((
        input,
        LgpFileHeader {
            filename,
            byte_size,
        },
    ))
}

fn lgp_product_name(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, _) = tag(b"FINAL FANTASY7")(input)?;
    Ok((input, ()))
}

pub fn parse_lgp<T>(mut reader: T) -> Result<Lgp<T>>
where
    T: Read + Seek,
{
    let mut input = [0u8; LGP_HEADER_SIZE];
    reader.read_exact(&mut input)?;
    let (_, lgp_header) = lgp_header(&input).map_err(|e| e.to_owned())?;

    let mut input: Vec<u8> = Vec::with_capacity(lgp_header.file_count);
    reader
        .by_ref()
        .take((lgp_header.file_count * LGP_TOC_SIZE).try_into()?)
        .read_to_end(&mut input)?;
    let (_, lgp_toc) = lgp_toc(&input, lgp_header.file_count).map_err(|e| e.to_owned())?;

    let mut input = [0u8; LGP_LOOKUP_TABLE_SIZE * LGP_LOOKUP_TABLE_SIZE * 4];
    reader.read_exact(&mut input)?;
    let (_, lgp_lookup_table) = lgp_lookup_table(&input).map_err(|e| e.to_owned())?;

    reader.seek(SeekFrom::End(0))?;
    let lgp_size = reader.stream_position()?;
    reader.seek(SeekFrom::Start(lgp_size - LGP_PRODUCT_NAME_SIZE as u64))?;

    let mut input = [0u8; LGP_PRODUCT_NAME_SIZE];
    reader.read_exact(&mut input)?;
    lgp_product_name(&input).map_err(|e| e.to_owned())?;

    reader.rewind()?;

    Ok(Lgp {
        reader,
        toc: lgp_toc,
        lookup_table: lgp_lookup_table,
        byte_size: lgp_size.try_into()?,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn test() {
        let file = File::open("data/moviecam.lgp").unwrap();
        let buf_reader = BufReader::new(file);
        let mut lgp = parse_lgp(buf_reader).unwrap();

        println!("{:?}", lgp.get_file("biskdead.cam").unwrap());
    }
}
