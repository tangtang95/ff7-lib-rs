mod lgp_data;

pub mod lgp;
pub mod parser;

const LGP_HEADER_SIZE: usize = 16;
const LGP_TOC_FILENAME_SIZE: usize = 20;
const LGP_TOC_SIZE: usize = 27;
const LGP_LOOKUP_TABLE_SIZE: usize = 30;
const LGP_FILE_HEADER_SIZE: usize = 24;
const LGP_PRODUCT_NAME_SIZE: usize = 14;
