pub use penumbra_macros::{FromBytes, ToBytes};

pub trait ToBytes {
    const SIZE: usize;

    type Output;
    fn to_bytes(&self) -> Self::Output;
}

pub trait FromBytes: Sized {
    const SIZE: usize;
    fn from_bytes(raw: &[u8]) -> Option<Self>;
}
