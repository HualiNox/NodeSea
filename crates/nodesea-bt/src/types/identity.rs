//! Domain identity types used by the BitTorrent and DHT layers.

use std::fmt::{self, Display};
use std::str::FromStr;

macro_rules! impl_hash_id {
    ($name:ident, $len:expr) => {
        impl $name {
            /// Length of the identifier in bytes.
            ///
            /// # Returns
            ///
            /// - `usize` - The fixed byte length of the identifier.
            pub const LEN: usize = $len;

            /// Creates the identifier from its fixed-size byte representation.
            ///
            /// # Arguments
            ///
            /// - `bytes` (`[u8; $len]`) - The raw identifier bytes.
            ///
            /// # Returns
            ///
            /// - `Self` - An identifier containing `bytes`.
            ///
            /// # Examples
            ///
            /// ```
            /// # use nodesea_bt::InfoHashV1;
            /// let _id = InfoHashV1::from_bytes([0; 20]);
            /// ```
            pub const fn from_bytes(bytes: [u8; $len]) -> Self {
                Self(bytes)
            }

            /// Returns the identifier's fixed-size byte representation.
            ///
            /// # Arguments
            ///
            /// - `&self` (`&Self`) - The identifier to inspect.
            ///
            /// # Returns
            ///
            /// - `&[u8; $len]` - The raw identifier bytes.
            pub const fn as_bytes(&self) -> &[u8; $len] {
                &self.0
            }

            /// Returns the identifier as a hexadecimal string.
            ///
            /// # Arguments
            ///
            /// - `&self` (`&Self`) - The identifier to format.
            ///
            /// # Returns
            ///
            /// - `String` - The lowercase hexadecimal representation.
            pub fn to_hex(&self) -> String {
                hex::encode(self.as_bytes())
            }

            /// Parses the identifier from a hexadecimal string.
            ///
            /// # Arguments
            ///
            /// - `value` (`&str`) - A hexadecimal string with exactly
            ///   `$len * 2` characters.
            ///
            /// # Returns
            ///
            /// - `Result<Self, hex::FromHexError>` - The parsed identifier, or
            ///   an error when the input is invalid.
            ///
            /// # Examples
            ///
            /// ```
            /// # use nodesea_bt::InfoHashV1;
            /// let _id = InfoHashV1::from_hex(
            ///     "0000000000000000000000000000000000000000",
            /// );
            /// ```
            pub fn from_hex(value: &str) -> Result<Self, hex::FromHexError> {
                let mut bytes = [0u8; $len];
                hex::decode_to_slice(value, &mut bytes)?;
                Ok(Self::from_bytes(bytes))
            }
        }

        impl FromStr for $name {
            type Err = hex::FromHexError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::from_hex(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = hex::FromHexError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::from_hex(value)
            }
        }

        impl TryFrom<&[u8]> for $name {
            type Error = std::array::TryFromSliceError;

            fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                Ok(Self::from_bytes(value.try_into()?))
            }
        }

        impl AsRef<[u8; $len]> for $name {
            fn as_ref(&self) -> &[u8; $len] {
                self.as_bytes()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}({})", stringify!($name), self.to_hex())
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.to_hex())
            }
        }
    };
}

/// A v1 torrent infohash represented by a 20-byte SHA-1 digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct InfoHashV1([u8; 20]);

impl_hash_id!(InfoHashV1, 20);

/// A v2 torrent infohash represented by a 32-byte SHA-256 digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct InfoHashV2([u8; 32]);

impl_hash_id!(InfoHashV2, 32);

/// A 20-byte DHT node identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct NodeId([u8; 20]);

impl_hash_id!(NodeId, 20);

/// A 20-byte DHT infohash key.
///
/// This may be a v1 SHA-1 infohash or a DHT representation derived from a
/// v2 SHA-256 infohash. The DHT wire representation does not identify which.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DhtInfoHash([u8; 20]);

impl_hash_id!(DhtInfoHash, 20);

/// A 20-byte DHT key-space traversal target.
///
/// In a BEP 51 sample request, this directs traversal through the DHT key
/// space. It is distinct from a local or remote [`NodeId`] and does not affect
/// the samples returned by the queried node.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DhtTarget([u8; 20]);

impl_hash_id!(DhtTarget, 20);

/// The v1 and/or v2 hash identity of one torrent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TorrentId {
    v1: Option<InfoHashV1>,
    v2: Option<InfoHashV2>,
}

impl TorrentId {
    /// Creates a torrent identity from its available hashes.
    ///
    /// # Arguments
    ///
    /// - `v1` (`Option<InfoHashV1>`) - An optional v1 infohash.
    /// - `v2` (`Option<InfoHashV2>`) - An optional v2 infohash.
    ///
    /// # Returns
    ///
    /// - `Self` - A torrent identity containing the supplied hashes.
    ///
    /// # Examples
    ///
    /// ```
    /// use nodesea_bt::{InfoHashV1, TorrentId};
    ///
    /// let v1 = InfoHashV1::from_bytes([0; 20]);
    /// let _id = TorrentId::new(Some(v1), None);
    /// ```
    pub const fn new(v1: Option<InfoHashV1>, v2: Option<InfoHashV2>) -> Self {
        Self { v1, v2 }
    }

    /// Returns the v1 hash, if present.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `Option<InfoHashV1>` - The v1 hash, if present.
    pub const fn v1(&self) -> Option<InfoHashV1> {
        self.v1
    }

    /// Returns the v2 hash, if present.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `Option<InfoHashV2>` - The v2 hash, if present.
    pub const fn v2(&self) -> Option<InfoHashV2> {
        self.v2
    }

    /// Returns whether a v1 hash is present.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `bool` - Whether a v1 hash is present.
    pub const fn has_v1(&self) -> bool {
        self.v1.is_some()
    }

    /// Returns whether a v2 hash is present.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `bool` - Whether a v2 hash is present.
    pub const fn has_v2(&self) -> bool {
        self.v2.is_some()
    }

    /// Returns whether neither hash is present.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `bool` - Whether neither a v1 nor v2 hash is present.
    pub const fn is_empty(&self) -> bool {
        self.v1.is_none() && self.v2.is_none()
    }

    /// Returns whether this is a v1-only identity.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `bool` - Whether only a v1 hash is present.
    pub const fn is_v1(&self) -> bool {
        self.v1.is_some() && self.v2.is_none()
    }

    /// Returns whether this is a v2-only identity.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `bool` - Whether only a v2 hash is present.
    pub const fn is_v2(&self) -> bool {
        self.v1.is_none() && self.v2.is_some()
    }

    /// Returns whether both v1 and v2 hashes are present.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The torrent identity to inspect.
    ///
    /// # Returns
    ///
    /// - `bool` - Whether both v1 and v2 hashes are present.
    pub const fn is_hybrid(&self) -> bool {
        self.v1.is_some() && self.v2.is_some()
    }
}

impl Display for TorrentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.v1, self.v2) {
            (Some(v1), Some(v2)) => write!(f, "Hybrid({}/{})", v1.to_hex(), v2.to_hex()),
            (Some(v1), None) => write!(f, "V1({})", v1.to_hex()),
            (None, Some(v2)) => write!(f, "V2({})", v2.to_hex()),
            (None, None) => write!(f, "Empty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_hash_hex_roundtrip() {
        let hex_str = "0123456789abcdef0123456789abcdef01234567";
        let hash = InfoHashV1::from_hex(hex_str).expect("Valid 40-character hex string");
        assert_eq!(hash.to_hex(), hex_str);
        assert_eq!(format!("{hash}"), hex_str);
        assert_eq!(format!("{hash:?}"), format!("InfoHashV1({hex_str})"));
    }

    #[test]
    fn test_info_hash_traits() {
        let hex_str = "0123456789abcdef0123456789abcdef01234567";
        let hash: InfoHashV1 = hex_str.parse().expect("Parse via FromStr");
        assert_eq!(hash.to_hex(), hex_str);

        let hash_try_str = InfoHashV1::try_from(hex_str).expect("TryFrom &str");
        assert_eq!(hash_try_str, hash);

        let raw = [7u8; 20];
        let hash_try_slice = InfoHashV1::try_from(&raw[..]).expect("TryFrom &[u8]");
        assert_eq!(hash_try_slice.as_bytes(), &raw);
    }

    #[test]
    fn test_info_hash_invalid_hex() {
        assert!(InfoHashV1::from_hex("invalid_hex_characters_here!!").is_err());
        assert!(InfoHashV1::from_hex("12345678").is_err());
    }

    #[test]
    fn test_info_hash_from_bytes() {
        let raw_bytes = [42u8; 20];
        let hash = InfoHashV1::from_bytes(raw_bytes);
        assert_eq!(hash.as_bytes(), &raw_bytes);
    }

    #[test]
    fn test_torrent_id_v2_and_hybrid_identity() {
        let v1 = InfoHashV1::from_bytes([0x11; 20]);
        let v2 = InfoHashV2::from_bytes([0x22; 32]);

        let v2_id = TorrentId::new(None, Some(v2));
        assert!(v2_id.is_v2());
        assert!(!v2_id.has_v1());
        assert_eq!(v2_id.v2(), Some(v2));

        let hybrid_id = TorrentId::new(Some(v1), Some(v2));
        assert!(hybrid_id.is_hybrid());
        assert!(hybrid_id.has_v1());
        assert!(hybrid_id.has_v2());
        assert_eq!(hybrid_id.v1(), Some(v1));
        assert_eq!(hybrid_id.v2(), Some(v2));
    }

    #[test]
    fn test_info_hash_default_and_order() {
        let default_hash = InfoHashV1::default();
        assert_eq!(default_hash.as_bytes(), &[0u8; 20]);
        assert_eq!(default_hash, InfoHashV1::from_bytes([0u8; 20]));

        let hash_a = InfoHashV1::from_bytes([1u8; 20]);
        let hash_b = InfoHashV1::from_bytes([2u8; 20]);
        assert!(hash_a < hash_b);
        assert_eq!(hash_a.as_ref(), &[1u8; 20]);

        let invalid_slice = [0u8; 19];
        assert!(InfoHashV1::try_from(&invalid_slice[..]).is_err());
    }
}
