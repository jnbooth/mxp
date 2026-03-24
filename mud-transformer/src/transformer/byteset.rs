use std::iter::FusedIterator;
use std::ops::{Not, RangeInclusive};
use std::{fmt, ptr};

#[repr(transparent)]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ByteSet {
    bytes: [u8; 32],
}

impl ByteSet {
    pub const fn new() -> Self {
        Self { bytes: [0; 32] }
    }

    const fn from_u64s(u64s: [u64; 4]) -> Self {
        // SAFETY: Identical size and layout.
        unsafe { std::mem::transmute(u64s) }
    }

    const fn into_u64s(self) -> [u64; 4] {
        // SAFETY: Identical size and layout.
        unsafe { std::mem::transmute(self) }
    }

    const fn as_u64s(&self) -> &[u64; 4] {
        // SAFETY: Identical size and layout.
        unsafe { &*(ptr::from_ref(self).cast()) }
    }

    const fn as_u64s_mut(&mut self) -> &mut [u64; 4] {
        // SAFETY: Identical size and layout.
        unsafe { &mut *(ptr::from_mut(self).cast()) }
    }

    pub const fn is_empty(&self) -> bool {
        let [a, b, c, d] = *self.as_u64s();
        a == 0 && b == 0 && c == 0 && d == 0
    }

    pub const fn len(&self) -> usize {
        let [a, b, c, d] = self.as_u64s();
        (a.count_ones() + b.count_ones() + c.count_ones() + d.count_ones()) as usize
    }

    pub const fn clear(&mut self) {
        *self = Self::new();
    }

    pub const fn contains(&self, i: u8) -> bool {
        self.byte(i) & Self::bit(i) != 0
    }

    pub const fn insert(&mut self, i: u8) {
        *self.byte_mut(i) |= Self::bit(i);
    }

    pub const fn remove(&mut self, i: u8) {
        *self.byte_mut(i) &= !Self::bit(i);
    }

    pub fn iter(&self) -> ByteSetIter<'_> {
        self.into_iter()
    }

    const fn byte(&self, i: u8) -> u8 {
        self.bytes[i as usize >> 3]
    }

    const fn byte_mut(&mut self, i: u8) -> &mut u8 {
        &mut self.bytes[i as usize >> 3]
    }

    const fn bit(i: u8) -> u8 {
        1 << (i & 7)
    }
}

impl fmt::Debug for ByteSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl Not for ByteSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        let [a, b, c, d] = self.into_u64s();
        Self::from_u64s([!a, !b, !c, !d])
    }
}

macro_rules! impl_bitassign {
    ($i:ident, $f:ident) => {
        impl std::ops::$i for ByteSet {
            fn $f(&mut self, rhs: Self) {
                let [l1, l2, l3, l4] = self.as_u64s_mut();
                let [r1, r2, r3, r4] = rhs.into_u64s();
                l1.$f(r1);
                l2.$f(r2);
                l3.$f(r3);
                l4.$f(r4);
            }
        }
    };
}

impl_bitassign!(BitOrAssign, bitor_assign);
impl_bitassign!(BitAndAssign, bitand_assign);
impl_bitassign!(BitXorAssign, bitxor_assign);

macro_rules! impl_bit {
    ($i:ident, $f:ident) => {
        impl std::ops::$i for ByteSet {
            type Output = Self;

            fn $f(self, rhs: Self) -> Self {
                let [l1, l2, l3, l4] = self.into_u64s();
                let [r1, r2, r3, r4] = rhs.into_u64s();
                Self::from_u64s([l1.$f(r1), l2.$f(r2), l3.$f(r3), l4.$f(r4)])
            }
        }
    };
}

impl_bit!(BitOr, bitor);
impl_bit!(BitAnd, bitand);
impl_bit!(BitXor, bitxor);

impl Extend<u8> for ByteSet {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl FromIterator<u8> for ByteSet {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

impl<'a> IntoIterator for &'a ByteSet {
    type Item = u8;
    type IntoIter = ByteSetIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ByteSetIter {
            set: self,
            inner: 0..=255,
        }
    }
}

pub struct ByteSetIter<'a> {
    set: &'a ByteSet,
    inner: RangeInclusive<u8>,
}

impl Iterator for ByteSetIter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|&byte| self.set.contains(byte))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            0,
            Some(((self.inner.end() - self.inner.start()) as usize + 1).min(self.inner.len())),
        )
    }

    fn min(mut self) -> Option<u8> {
        self.next()
    }

    fn max(mut self) -> Option<u8> {
        self.next_back()
    }

    fn is_sorted(self) -> bool {
        true
    }
}

impl DoubleEndedIterator for ByteSetIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.rfind(|&byte| self.set.contains(byte))
    }
}

impl FusedIterator for ByteSetIter<'_> {}
