use std::io::{self, Read};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

/// A reader that generates zero bytes for upload testing
pub struct ZeroReader {
    remaining: usize,
    written: usize,
}

impl ZeroReader {
    /// Create a new zero reader with the specified size
    pub fn new(size: usize) -> Self {
        Self {
            remaining: size,
            written: 0,
        }
    }

    /// Get the number of bytes written so far
    pub fn written_bytes(&self) -> usize {
        self.written
    }

    /// Get the total size
    pub fn total_size(&self) -> usize {
        self.written + self.remaining
    }
}

impl Read for ZeroReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Ok(0);
        }

        let to_read = std::cmp::min(buf.len(), self.remaining);
        buf[..to_read].fill(0);

        self.remaining -= to_read;
        self.written += to_read;

        Ok(to_read)
    }
}

impl AsyncRead for ZeroReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.remaining == 0 {
            return Poll::Ready(Ok(()));
        }

        let to_read = std::cmp::min(buf.remaining(), self.remaining);
        let zeros = vec![0u8; to_read];
        buf.put_slice(&zeros);

        self.remaining -= to_read;
        self.written += to_read;

        Poll::Ready(Ok(()))
    }
}

// Implement reqwest::Body conversion for upload testing
impl From<ZeroReader> for reqwest::Body {
    fn from(reader: ZeroReader) -> Self {
        reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(reader))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_zero_reader() {
        let mut reader = ZeroReader::new(100);
        let mut buffer = [1u8; 50];

        // First read
        let bytes_read = reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 50);
        assert_eq!(buffer, [0u8; 50]);
        assert_eq!(reader.written_bytes(), 50);

        // Second read
        let bytes_read = reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 50);
        assert_eq!(buffer, [0u8; 50]);
        assert_eq!(reader.written_bytes(), 100);

        // Third read (EOF)
        let bytes_read = reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 0);
        assert_eq!(reader.written_bytes(), 100);
    }

    #[test]
    fn test_zero_reader_partial() {
        let mut reader = ZeroReader::new(75);
        let mut buffer = [1u8; 100];

        let bytes_read = reader.read(&mut buffer).unwrap();
        assert_eq!(bytes_read, 75);
        assert_eq!(&buffer[..75], &[0u8; 75]);
        assert_eq!(reader.written_bytes(), 75);
    }
}
