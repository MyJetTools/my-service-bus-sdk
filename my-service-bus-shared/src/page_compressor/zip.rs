use std::io::{Cursor, Read, Write};

use zip::result::ZipError;

use super::vec_writer::VecWriter;

pub fn decompress_payload(payload: &[u8]) -> Result<Vec<u8>, ZipError> {
    let c = Cursor::new(payload.to_vec());

    let mut zip = zip::ZipArchive::new(c)?;

    let mut page_buffer: Vec<u8> = Vec::new();

    for i in 0..zip.len() {
        let mut zip_file = zip.by_index(i)?;

        if zip_file.name() == "d" {
            let mut buffer = [0u8; 1024 * 1024];

            loop {
                let read_size = zip_file.read(&mut buffer[..])?;
                if read_size == 0 {
                    break;
                }

                page_buffer.extend(&buffer[..read_size]);
            }
        }
    }

    Ok(page_buffer)
}

pub fn compress_payload(payload: &[u8]) -> Result<Vec<u8>, ZipError> {
    let mut writer = VecWriter::new();

    {
        let mut zip = zip::ZipWriter::new(&mut writer);

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("d", options)?;

        let mut pos = 0;
        while pos < payload.len() {
            let size = zip.write(&payload[pos..])?;

            pos += size;
        }

        zip.finish()?;
    }

    Ok(writer.buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_unzip() {
        let src = vec![0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8];

        let compressed = compress_payload(&src).unwrap();

        println!("{}", compressed.len());

        let uncompressed = decompress_payload(&compressed).unwrap();

        println!("{}", uncompressed.len());
    }
}
