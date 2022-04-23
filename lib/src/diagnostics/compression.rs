use std::io::{Error, ErrorKind, Read, Result};

use flate2::read::ZlibDecoder;

use crate::bytes;

pub(crate) fn decompress<R: Read + ?Sized>(reader: &mut R) -> Result<Vec<u8>> {
    let buffer_size: usize = bytes::read_le_u32(reader)?
        .try_into()
        .map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
    let mut decoder = ZlibDecoder::new(reader);
    let mut buffer = Vec::with_capacity(buffer_size);

    decoder.read_to_end(&mut buffer)?;

    Ok(buffer)
}
