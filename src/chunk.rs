use crate::chunk_type::ChunkType;
use crc::{Algorithm, Crc, CRC_32_ISO_HDLC};
use std::fmt::{Display, Formatter};

pub struct Chunk {
    typ: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

const CRC_32_POLY: u32 = 0x04C11DB7;
const CRC_32_ALGO: Algorithm<u32> = Algorithm {
    poly: CRC_32_POLY,
    init: 0xFFFFFFFF,
    refin: true,
    refout: true,
    xorout: 0xFFFFFFFF,
    check: 0xCBF43926,
    residue: 0x00000000,
    width: 32,
};

impl TryFrom<&[u8]> for Chunk {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 4 {
            return Err(());
        }

        let mut len = [0u8; 4];
        len.copy_from_slice(&value[..4]);
        let len = u32::from_be_bytes(len) as usize;
        let mut remain = &value[4..];
        if remain.len() < len + 8 {
            return Err(());
        }

        let mut typ = [0u8; 4];
        typ.copy_from_slice(&remain[..4]);
        remain = &remain[4..];
        let data = &remain[..len];
        remain = &remain[len..];
        let typ = ChunkType::try_from(typ)?;
        let chunk = Chunk::new(typ, data.to_vec());

        let mut crc = [0u8; 4];
        crc.copy_from_slice(remain);
        let crc = u32::from_be_bytes(crc);

        if chunk.crc != crc {
            return Err(());
        }

        Ok(chunk)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.data_as_string() {
            Ok(v) => write!(f, "{}", v),
            Err(e) => write!(f, "Error: {}", e),
        }
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc32 = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let mut digest = crc32.digest();
        digest.update(&chunk_type.bytes());
        digest.update(&data);
        let crc = digest.finalize();

        Self {
            typ: chunk_type,
            data,
            crc,
        }
    }
    fn length(&self) -> u32 {
        self.data.len() as u32
    }
    fn chunk_type(&self) -> &ChunkType {
        &self.typ
    }
    fn data(&self) -> &[u8] {
        &self.data
    }
    fn crc(&self) -> u32 {
        self.crc
    }
    fn data_as_string(&self) -> crate::Result<String> {
        Ok(String::from_utf8_lossy(&self.data).to_string())
    }
    fn as_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
