//! Domain identity types used by the BitTorrent and DHT layers.

use std::fmt::{self, Display};
use std::str::FromStr;

macro_rules! impl_hash_id {
    ($name:ident, $len:expr) => {
        impl $name {
            /// Length of the identifier in bytes.
            pub const LEN: usize = $len;

            /// Creates the identifier from its fixed-size byte representation.
            pub const fn from_bytes(bytes: [u8; $len]) -> Self {
                Self(bytes)
            }

            /// Returns the identifier's fixed-size byte representation.
            pub const fn as_bytes(&self) -> &[u8; $len] {
                &self.0
            }

            /// Returns the identifier as a hexadecimal string.
            pub fn to_hex(&self) -> String {
                hex::encode(self.as_bytes())
            }

            /// Parses the identifier from a hexadecimal string.
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

        impl From<[u8; $len]> for $name {
            fn from(value: [u8; $len]) -> Self {
                Self::from_bytes(value)
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
    pub const fn new(v1: Option<InfoHashV1>, v2: Option<InfoHashV2>) -> Self {
        Self { v1, v2 }
    }

    /// Returns the v1 hash, if present.
    pub const fn v1(&self) -> Option<InfoHashV1> {
        self.v1
    }

    /// Returns the v2 hash, if present.
    pub const fn v2(&self) -> Option<InfoHashV2> {
        self.v2
    }

    /// Returns whether a v1 hash is present.
    pub const fn has_v1(&self) -> bool {
        self.v1.is_some()
    }

    /// Returns whether a v2 hash is present.
    pub const fn has_v2(&self) -> bool {
        self.v2.is_some()
    }

    /// Returns whether neither hash is present.
    pub const fn is_empty(&self) -> bool {
        self.v1.is_none() && self.v2.is_none()
    }

    /// Returns whether this is a v1-only identity.
    pub const fn is_v1(&self) -> bool {
        self.v1.is_some() && self.v2.is_none()
    }

    /// Returns whether this is a v2-only identity.
    pub const fn is_v2(&self) -> bool {
        self.v1.is_none() && self.v2.is_some()
    }

    /// Returns whether both v1 and v2 hashes are present.
    pub const fn is_hybrid(&self) -> bool {
        self.v1.is_some() && self.v2.is_some()
    }
}

impl From<InfoHashV1> for TorrentId {
    fn from(info_hash: InfoHashV1) -> Self {
        Self::new(Some(info_hash), None)
    }
}

impl From<InfoHashV2> for TorrentId {
    fn from(info_hash: InfoHashV2) -> Self {
        Self::new(None, Some(info_hash))
    }
}

impl From<[u8; 20]> for TorrentId {
    fn from(bytes: [u8; 20]) -> Self {
        Self::new(Some(InfoHashV1::from_bytes(bytes)), None)
    }
}

impl From<&[u8; 20]> for TorrentId {
    fn from(bytes: &[u8; 20]) -> Self {
        Self::new(Some(InfoHashV1::from_bytes(*bytes)), None)
    }
}

impl From<[u8; 32]> for TorrentId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(None, Some(InfoHashV2::from_bytes(bytes)))
    }
}

impl From<&[u8; 32]> for TorrentId {
    fn from(bytes: &[u8; 32]) -> Self {
        Self::new(None, Some(InfoHashV2::from_bytes(*bytes)))
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
