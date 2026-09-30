use tokio::io::{self, AsyncRead, AsyncReadExt};

use crate::io::MAGIC_NUMBER;

pub(super) async fn read_magic_number<R>(reader: &mut R) -> io::Result<()>
where
    R: AsyncRead + Unpin,
{
    let mut buf = [0; MAGIC_NUMBER.len()];
    reader.read_exact(&mut buf).await?;

    if buf == MAGIC_NUMBER {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid magic number: expected {MAGIC_NUMBER:#04x?}, got {buf:#04x?}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_magic_number() {
        let data = b"CSI\x01";
        let mut reader = &data[..];
        assert!(read_magic_number(&mut reader).await.is_ok());

        let data = [];
        let mut reader = &data[..];
        assert!(matches!(
            read_magic_number(&mut reader).await,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof
        ));

        let data = b"MThd";
        let mut reader = &data[..];
        assert!(matches!(
            read_magic_number(&mut reader).await,
            Err(ref e) if e.kind() == io::ErrorKind::InvalidData
        ));
    }
}
