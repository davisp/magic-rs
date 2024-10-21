pub(crate) trait ReadLittleEndian {
    fn read_le(bytes: &[u8]) -> Self;
}

macro_rules! impl_from_le_bytes (($($type:ty), *) => {
    $(
        impl ReadLittleEndian for $type {
            fn read_le(bytes: &[u8]) -> Self {
                assert!(bytes.len() >= std::mem::size_of::<Self>());
                let mut value = [0u8; std::mem::size_of::<Self>()];
                value.copy_from_slice(&bytes[0..std::mem::size_of::<Self>()]);
                Self::from_le_bytes(value)
            }
        }
    )*
});

impl_from_le_bytes!(u8, u16, u32, u64, i8, i16, i32, i64);
