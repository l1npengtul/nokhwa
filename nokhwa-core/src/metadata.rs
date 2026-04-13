use std::{collections::HashSet, fmt::Display, hash::Hash};

pub use time::Time;
pub use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataValue {
    String(String),
    Uuid(Uuid),
    I64(i64),
    U64(u64),
    I32(i32),
    U32(u32),
    Byte(u8),
    Timestamp(Time),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetadataTypes {
    Size(u64),
    Timestamp(Time),
    PlatformBitflag32(u32),
    PlatformBitflag64(u64),
    Sequence(u64),
    FieldOrder(u32),
    Custom { name: String, value: MetadataValue },
}

impl Hash for MetadataTypes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            MetadataTypes::Custom { name, .. } => name.hash(state),
            meta => core::mem::discriminant(meta).hash(state),
        }
    }
}

impl Display for MetadataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataTypes::Size(s) => write!(f, "Metadata Size: {s}b"),
            MetadataTypes::Timestamp(time) => {
                let (hour, minute, second, micro) = time.as_hms_micro();
                write!(
                    f,
                    "Metadata Timestamp: {hour}h {minute}m {second}s {micro}us"
                )
            }
            MetadataTypes::PlatformBitflag32(bf) => write!(f, "Metadata Bitflag 32bit: {bf:032b}"),
            MetadataTypes::PlatformBitflag64(bf) => write!(f, "Metadata Bitflag 32bit: {bf:064b}"),
            MetadataTypes::Sequence(seq) => write!(f, "Metadata Order in Sequence: #{seq}"),
            MetadataTypes::FieldOrder(or) => write!(f, "Metadata Field Order: {or}"),
            MetadataTypes::Custom { name, value } => write!(f, "Metadata {name}: {value:?}"),
        }
    }
}

pub type Metadata = HashSet<MetadataTypes>;
