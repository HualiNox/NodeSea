//! Domain identity types used by the BitTorrent and DHT layers.

use std::fmt::{self, Display};
use std::str::FromStr;

macro_rules! impl_hash_id {
    ($name:ident) => {
        impl $name {
            /// Creates the identifier from its 20-byte representation.
            pub const fn from_bytes(bytes: [u8; 20]) -> Self {
                Self(bytes)
            }

            /// Returns the identifier's 20-byte representation.
            pub const fn as_bytes(&self) -> &[u8; 20] {
                &self.0
            }

            /// Returns the identifier as a hexadecimal string.
            pub fn to_hex(&self) -> String {
                hex::encode(self.as_bytes())
            }

            /// Parses the identifier from a hexadecimal string.
            pub fn from_hex(value: &str) -> Result<Self, hex::FromHexError> {
                let mut bytes = [0u8; 20];
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

        impl From<[u8; 20]> for $name {
            fn from(value: [u8; 20]) -> Self {
                Self::from_bytes(value)
            }
        }

        impl AsRef<[u8; 20]> for $name {
            fn as_ref(&self) -> &[u8; 20] {
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

/// A 20-byte torrent identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct InfoHash([u8; 20]);

impl_hash_id!(InfoHash);

/// A 20-byte DHT node identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct NodeId([u8; 20]);

impl_hash_id!(NodeId);

/// A 20-byte DHT key-space traversal target.
///
/// In a BEP 51 sample request, this directs traversal through the DHT key
/// space. It is distinct from a local or remote [`NodeId`] and does not affect
/// the samples returned by the queried node.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DhtTarget([u8; 20]);

impl_hash_id!(DhtTarget);
