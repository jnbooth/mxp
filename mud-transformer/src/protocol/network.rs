/// Types that can be converted to byte arrays in big-endian order.
pub trait ToBeBytes: Copy {
    /// Resulting array type. Should be `[u8; N]`.
    type BeBytes;

    /// Converts to a big-endian byte array.
    fn to_be_bytes(self) -> Self::BeBytes;
}

impl<C: ToBeBytes> ToBeBytes for &C {
    type BeBytes = C::BeBytes;

    #[inline]
    fn to_be_bytes(self) -> Self::BeBytes {
        C::to_be_bytes(*self)
    }
}

macro_rules! impl_to_be_bytes {
    ($t:ty, $n:literal) => {
        impl ToBeBytes for $t {
            type BeBytes = [u8; $n];

            #[inline]
            fn to_be_bytes(self) -> Self::BeBytes {
                self.to_be_bytes()
            }
        }
    };
}

impl_to_be_bytes!(u8, 1);
impl_to_be_bytes!(u16, 2);
impl_to_be_bytes!(u32, 4);
impl_to_be_bytes!(u64, 8);

impl ToBeBytes for char {
    type BeBytes = [u8; 4];

    #[inline]
    fn to_be_bytes(self) -> Self::BeBytes {
        u32::from(self).to_be_bytes()
    }
}
