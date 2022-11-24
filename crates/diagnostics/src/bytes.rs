use std::io::{Error, ErrorKind, Read, Result};

/// Read a u8 value from a reader.
pub(crate) fn read_u8<R: Read + ?Sized>(reader: &mut R) -> Result<u8> {
    let mut buf = [0u8];
    reader.read_exact(&mut buf)?;

    Ok(buf[0])
}

/// Read a u32 value from a reader in little endian.
pub(crate) fn read_le_u32<R: Read + ?Sized>(reader: &mut R) -> Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;

    Ok(u32::from_le_bytes(bytes))
}

/// Read a variable-byte encoded u64.
pub(crate) fn read_var_u64<R: Read + ?Sized>(reader: &mut R) -> Result<u64> {
    let mut cumulative_value: u64 = 0;
    let mut bytes_read = 0;
    let mut shifted_bits: u32 = 0;

    loop {
        if bytes_read > 9 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Maximum bytes size for a variable u64 has been reached.".to_string(),
            ));
        }

        let byte = read_u8(reader)?;
        if byte < 128 {
            return Ok(cumulative_value + ((byte as u64) << shifted_bits));
        }

        cumulative_value += ((byte & 127) as u64) << shifted_bits;
        shifted_bits += 7;
        bytes_read += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_var_u64_reads_one_byte() {
        let expected_value: u64 = 127;
        let bytes = [0x7f];
        let mut cursor = Cursor::new(bytes);

        let actual_value = read_var_u64(&mut cursor).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn read_var_u64_reads_two_bytes() {
        let expected_value: u64 = 255;
        let bytes = [0xff, 0x1];
        let mut cursor = Cursor::new(bytes);

        let actual_value = read_var_u64(&mut cursor).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    #[test]
    #[should_panic(expected = "Maximum bytes size for a variable u64 has been reached.")]
    fn read_var_u64_returns_error_when_reading_more_than_ten_bytes() {
        let bytes = [0xff; 10];
        let mut cursor = Cursor::new(bytes);

        read_var_u64(&mut cursor).unwrap();
    }
}
