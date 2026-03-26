use std::fmt;
use std::iter::FusedIterator;
use std::ops::{Index, Not, RangeInclusive};

#[repr(transparent)]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ByteSet {
    bytes: [u64; 4],
}

impl ByteSet {
    pub const fn new() -> Self {
        Self { bytes: [0; 4] }
    }

    pub const fn is_empty(&self) -> bool {
        let [a, b, c, d] = self.bytes;
        a == 0 && b == 0 && c == 0 && d == 0
    }

    pub const fn len(&self) -> usize {
        let [a, b, c, d] = self.bytes;
        (a.count_ones() + b.count_ones() + c.count_ones() + d.count_ones()) as usize
    }

    pub const fn clear(&mut self) {
        *self = Self::new();
    }

    pub const fn contains(&self, i: u8) -> bool {
        let (high, low) = Self::indices(i);
        self.bytes[high] & low != 0
    }

    pub const fn insert(&mut self, i: u8) {
        let (high, low) = Self::indices(i);
        self.bytes[high] |= low;
    }

    pub const fn remove(&mut self, i: u8) {
        let (high, low) = Self::indices(i);
        self.bytes[high] &= !low;
    }

    pub fn iter(&self) -> ByteSetIter<'_> {
        self.into_iter()
    }

    const fn indices(i: u8) -> (usize, u64) {
        (i as usize >> 6, 1 << (i & 63))
    }
}

impl fmt::Debug for ByteSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl Index<u8> for ByteSet {
    type Output = bool;

    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        if self.contains(index) { &true } else { &false }
    }
}

impl Not for ByteSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        let [a, b, c, d] = self.bytes;
        Self {
            bytes: [!a, !b, !c, !d],
        }
    }
}

macro_rules! impl_bitassign {
    ($i:ident, $f:ident) => {
        impl std::ops::$i for ByteSet {
            fn $f(&mut self, rhs: Self) {
                let [l1, l2, l3, l4] = &mut self.bytes;
                let [r1, r2, r3, r4] = rhs.bytes;
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
                let [l1, l2, l3, l4] = self.bytes;
                let [r1, r2, r3, r4] = rhs.bytes;
                Self {
                    bytes: [l1.$f(r1), l2.$f(r2), l3.$f(r3), l4.$f(r4)],
                }
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

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|&byte| self.set.contains(byte))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let range = self.inner.end() - self.inner.start();
        (0, Some((usize::from(range) + 1).min(self.set.len())))
    }

    #[inline]
    fn min(mut self) -> Option<u8> {
        self.next()
    }

    #[inline]
    fn max(mut self) -> Option<u8> {
        self.next_back()
    }

    #[inline]
    fn is_sorted(self) -> bool {
        true
    }
}

impl DoubleEndedIterator for ByteSetIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.rfind(|&byte| self.set.contains(byte))
    }
}

impl FusedIterator for ByteSetIter<'_> {}
