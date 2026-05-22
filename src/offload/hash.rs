use md5::{Digest as Md5Digest, Md5};
use sha1::Sha1;
use xxhash_rust::xxh3::{Xxh3, Xxh3Builder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChecksumAlgo {
    #[default]
    Blake3,
    Md5,
    Sha1,
    Xxh3_64,
    Xxh3_128,
}

impl ChecksumAlgo {
    pub fn as_str(self) -> &'static str {
        match self {
            ChecksumAlgo::Blake3 => "BLAKE3",
            ChecksumAlgo::Md5 => "MD5",
            ChecksumAlgo::Sha1 => "SHA1",
            ChecksumAlgo::Xxh3_64 => "XXH3-64",
            ChecksumAlgo::Xxh3_128 => "XXH3-128",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "BLAKE3" | "B3" => Some(ChecksumAlgo::Blake3),
            "MD5" => Some(ChecksumAlgo::Md5),
            "SHA1" | "SHA-1" => Some(ChecksumAlgo::Sha1),
            "XXH3" | "XXH3-64" | "XXH3_64" => Some(ChecksumAlgo::Xxh3_64),
            "XXH3-128" | "XXH3_128" => Some(ChecksumAlgo::Xxh3_128),
            _ => None,
        }
    }

    /// The MHL <hash> child element name to use in ASC MHL v2.0 output.
    pub fn mhl_element_name(self) -> &'static str {
        match self {
            ChecksumAlgo::Blake3 => "blake3",
            ChecksumAlgo::Md5 => "md5",
            ChecksumAlgo::Sha1 => "sha1",
            ChecksumAlgo::Xxh3_64 => "xxh3",
            ChecksumAlgo::Xxh3_128 => "xxh3-128",
        }
    }

    pub fn new_hasher(self) -> Box<dyn FileHasher> {
        match self {
            ChecksumAlgo::Blake3 => Box::new(Blake3Hasher::new()),
            ChecksumAlgo::Md5 => Box::new(Md5Hasher::new()),
            ChecksumAlgo::Sha1 => Box::new(Sha1Hasher::new()),
            ChecksumAlgo::Xxh3_64 => Box::new(Xxh3_64Hasher::new()),
            ChecksumAlgo::Xxh3_128 => Box::new(Xxh3_128Hasher::new()),
        }
    }
}

/// Streaming file hasher. Callers feed it bytes via update() and obtain
/// a lowercase-hex digest via finalize_hex().
pub trait FileHasher: Send {
    fn update(&mut self, bytes: &[u8]);
    fn finalize_hex(self: Box<Self>) -> String;
}

#[derive(Default)]
pub struct Blake3Hasher(blake3::Hasher);
impl Blake3Hasher {
    pub fn new() -> Self {
        Self::default()
    }
}
impl FileHasher for Blake3Hasher {
    fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
    fn finalize_hex(self: Box<Self>) -> String {
        self.0.finalize().to_hex().to_string()
    }
}

#[derive(Default)]
pub struct Md5Hasher(Md5);
impl Md5Hasher {
    pub fn new() -> Self {
        Self::default()
    }
}
impl FileHasher for Md5Hasher {
    fn update(&mut self, bytes: &[u8]) {
        Md5Digest::update(&mut self.0, bytes);
    }
    fn finalize_hex(self: Box<Self>) -> String {
        let out = self.0.finalize();
        hex_encode(&out)
    }
}

#[derive(Default)]
pub struct Sha1Hasher(Sha1);
impl Sha1Hasher {
    pub fn new() -> Self {
        Self::default()
    }
}
impl FileHasher for Sha1Hasher {
    fn update(&mut self, bytes: &[u8]) {
        sha1::Digest::update(&mut self.0, bytes);
    }
    fn finalize_hex(self: Box<Self>) -> String {
        let out = self.0.finalize();
        hex_encode(&out)
    }
}

pub struct Xxh3_64Hasher(Xxh3);
impl Default for Xxh3_64Hasher {
    fn default() -> Self {
        Self(Xxh3Builder::new().build())
    }
}
impl Xxh3_64Hasher {
    pub fn new() -> Self {
        Self::default()
    }
}
impl FileHasher for Xxh3_64Hasher {
    fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
    fn finalize_hex(self: Box<Self>) -> String {
        format!("{:016x}", self.0.digest())
    }
}

pub struct Xxh3_128Hasher(Xxh3);
impl Default for Xxh3_128Hasher {
    fn default() -> Self {
        Self(Xxh3Builder::new().build())
    }
}
impl Xxh3_128Hasher {
    pub fn new() -> Self {
        Self::default()
    }
}
impl FileHasher for Xxh3_128Hasher {
    fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
    fn finalize_hex(self: Box<Self>) -> String {
        format!("{:032x}", self.0.digest128())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{:02x}", b);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_str(algo: ChecksumAlgo, data: &[u8]) -> String {
        let mut h = algo.new_hasher();
        h.update(data);
        h.finalize_hex()
    }

    #[test]
    fn algo_parse_roundtrip() {
        for algo in [
            ChecksumAlgo::Blake3,
            ChecksumAlgo::Md5,
            ChecksumAlgo::Sha1,
            ChecksumAlgo::Xxh3_64,
            ChecksumAlgo::Xxh3_128,
        ] {
            assert_eq!(ChecksumAlgo::parse(algo.as_str()), Some(algo));
        }
    }

    #[test]
    fn algo_parse_aliases() {
        assert_eq!(ChecksumAlgo::parse("blake3"), Some(ChecksumAlgo::Blake3));
        assert_eq!(ChecksumAlgo::parse("b3"), Some(ChecksumAlgo::Blake3));
        assert_eq!(ChecksumAlgo::parse("md5"), Some(ChecksumAlgo::Md5));
        assert_eq!(ChecksumAlgo::parse("sha-1"), Some(ChecksumAlgo::Sha1));
        assert_eq!(ChecksumAlgo::parse("xxh3"), Some(ChecksumAlgo::Xxh3_64));
        assert_eq!(
            ChecksumAlgo::parse("xxh3-128"),
            Some(ChecksumAlgo::Xxh3_128)
        );
        assert_eq!(ChecksumAlgo::parse("nonsense"), None);
    }

    #[test]
    fn known_test_vectors() {
        // RFC test vectors / known reference outputs
        assert_eq!(
            hash_str(ChecksumAlgo::Md5, b""),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
        assert_eq!(
            hash_str(ChecksumAlgo::Sha1, b""),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        // BLAKE3 of empty string
        assert_eq!(
            hash_str(ChecksumAlgo::Blake3, b""),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
        // MD5 "abc" → 900150983cd24fb0d6963f7d28e17f72
        assert_eq!(
            hash_str(ChecksumAlgo::Md5, b"abc"),
            "900150983cd24fb0d6963f7d28e17f72"
        );
        // SHA1 "abc" → a9993e364706816aba3e25717850c26c9cd0d89d
        assert_eq!(
            hash_str(ChecksumAlgo::Sha1, b"abc"),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[test]
    fn streaming_matches_single_update() {
        for algo in [
            ChecksumAlgo::Blake3,
            ChecksumAlgo::Md5,
            ChecksumAlgo::Sha1,
            ChecksumAlgo::Xxh3_64,
            ChecksumAlgo::Xxh3_128,
        ] {
            let mut chunked = algo.new_hasher();
            chunked.update(b"hello ");
            chunked.update(b"world");
            let chunked_hex = chunked.finalize_hex();

            let mut whole = algo.new_hasher();
            whole.update(b"hello world");
            let whole_hex = whole.finalize_hex();

            assert_eq!(chunked_hex, whole_hex, "streaming mismatch for {:?}", algo);
        }
    }

    #[test]
    fn xxh3_64_width_is_16_hex_chars() {
        let s = hash_str(ChecksumAlgo::Xxh3_64, b"anything");
        assert_eq!(s.len(), 16);
    }

    #[test]
    fn xxh3_128_width_is_32_hex_chars() {
        let s = hash_str(ChecksumAlgo::Xxh3_128, b"anything");
        assert_eq!(s.len(), 32);
    }

    #[test]
    fn mhl_element_names_are_lowercase() {
        for algo in [
            ChecksumAlgo::Blake3,
            ChecksumAlgo::Md5,
            ChecksumAlgo::Sha1,
            ChecksumAlgo::Xxh3_64,
            ChecksumAlgo::Xxh3_128,
        ] {
            let name = algo.mhl_element_name();
            assert_eq!(name, name.to_ascii_lowercase());
            assert!(!name.is_empty());
        }
    }
}
