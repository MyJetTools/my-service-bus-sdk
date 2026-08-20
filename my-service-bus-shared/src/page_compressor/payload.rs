use std::io::Write;

use flate2::{write::ZlibEncoder, Compression, Decompress, FlushDecompress, Status};

/// Compresses a page payload as a single zlib (deflate) stream.
///
/// A page is one protobuf blob, so there is nothing to name inside it and no archive
/// container is involved. Zlib is picked over bare deflate because its 6 bytes of framing
/// carry an Adler-32 checksum, which lets `decompress_payload` tell a corrupted or
/// truncated blob from a valid one instead of handing garbage to the protobuf decoder.
pub fn compress_payload(payload: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(payload)?;
    encoder.finish()
}

/// Restores a payload written by [`compress_payload`].
///
/// Fails unless the stream reaches its end marker with a matching checksum. This is done
/// over `Decompress` rather than `flate2::read::ZlibDecoder`, because the reader reports
/// success once its input runs out even if the stream never ended - a blob whose tail got
/// cut off would decode to a silently short payload.
pub fn decompress_payload(compressed: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decompressor = Decompress::new(true);

    let mut result: Vec<u8> = Vec::with_capacity(compressed.len().max(256) * 4);

    loop {
        // decompress_vec only ever fills the spare capacity, so a full vec means the next
        // call would have nowhere to write and would spin on BufError.
        if result.len() == result.capacity() {
            result.reserve(result.capacity());
        }

        let consumed = decompressor.total_in() as usize;
        let produced = decompressor.total_out();

        let status = decompressor
            .decompress_vec(&compressed[consumed..], &mut result, FlushDecompress::Finish)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        match status {
            Status::StreamEnd => return Ok(result),
            Status::Ok | Status::BufError => {
                // Every byte is in and the stream still has not ended: the tail is missing.
                if decompressor.total_in() as usize == compressed.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "compressed payload ends before the end of the zlib stream",
                    ));
                }

                // There is input left and room to write, so a call that moved neither counter
                // cannot be made to move on the next one either - bail out instead of spinning.
                if decompressor.total_in() as usize == consumed
                    && decompressor.total_out() == produced
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "compressed payload stopped making progress before the stream ended",
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let src = b"Hello MyServiceBus page payload".repeat(100);

        let compressed = compress_payload(src.as_slice()).unwrap();

        assert!(compressed.len() < src.len());

        let uncompressed = decompress_payload(compressed.as_slice()).unwrap();

        assert_eq!(src.as_slice(), uncompressed.as_slice());
    }

    #[test]
    fn test_empty_payload() {
        let compressed = compress_payload(&[]).unwrap();

        let uncompressed = decompress_payload(compressed.as_slice()).unwrap();

        assert_eq!(0, uncompressed.len());
    }

    #[test]
    fn test_payload_which_does_not_compress() {
        // Incompressible input makes zlib fall back to stored blocks, which is the branch
        // where the output outgrows the input.
        let src: Vec<u8> = (0..8192u32).map(|i| i.wrapping_mul(2654435761).to_le_bytes()[0]).collect();

        let compressed = compress_payload(src.as_slice()).unwrap();

        let uncompressed = decompress_payload(compressed.as_slice()).unwrap();

        assert_eq!(src.as_slice(), uncompressed.as_slice());
    }

    #[test]
    fn test_payload_much_bigger_than_the_initial_buffer() {
        // 4Mb of highly compressible data: the compressed blob is a few Kb, so the output
        // buffer sized from it has to grow many times over while inflating.
        let src = b"MyServiceBus sub page message payload ".repeat(110_000);

        let compressed = compress_payload(src.as_slice()).unwrap();

        assert!(compressed.len() * 100 < src.len());

        let uncompressed = decompress_payload(compressed.as_slice()).unwrap();

        assert_eq!(src.len(), uncompressed.len());
        assert_eq!(src.as_slice(), uncompressed.as_slice());
    }

    #[test]
    fn test_missing_checksum_is_detected() {
        let src = b"Hello MyServiceBus page payload".repeat(100);

        let compressed = compress_payload(src.as_slice()).unwrap();

        // Adler-32 trailer only: the deflate stream itself is complete here.
        let result = decompress_payload(&compressed[..compressed.len() - 4]);

        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_truncated_stream_is_detected() {
        let src = b"Hello MyServiceBus page payload".repeat(100);

        let compressed = compress_payload(src.as_slice()).unwrap();

        let result = decompress_payload(&compressed[..compressed.len() / 2]);

        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_corrupted_stream_is_detected() {
        let src = b"Hello MyServiceBus page payload".repeat(100);

        let mut compressed = compress_payload(src.as_slice()).unwrap();

        let last = compressed.len() - 6;
        compressed[last] ^= 0xFF;

        let result = decompress_payload(compressed.as_slice());

        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_garbage_is_not_accepted() {
        let result = decompress_payload(&[0u8, 1u8, 2u8, 3u8]);

        assert_eq!(true, result.is_err());
    }
}
