pub trait BitSet: Sized {
    /// The type used to index into the bit set.
    type Index;

    /// Sets the bit at `index` to `true`.
    fn set(&mut self, index: Self::Index);

    /// Sets the bit at `index` to `false`.
    fn clear(&mut self, index: Self::Index);

    /// Sets all bits to `true`.
    fn set_all(&mut self);

    /// Sets all bits to `false`.
    fn clear_all(&mut self);

    /// Returns the bit at `index`.
    fn get(&self, index: Self::Index) -> bool;

    /// Returns the number of `true` bits in the set.
    fn len(&self) -> Self::Index;

    /// Returns `true` if the set contains no `true` bits.
    fn is_empty(&self) -> bool;

    /// Returns the difference, i.e. the bits that are in `self` but not in `other`.
    fn difference(&self, other: &Self) -> Self;

    /// Returns the symmetric difference, i.e. the bits that are in `self` or `other` but not in both.
    fn symmetric_difference(&self, other: &Self) -> Self;

    /// Returns the intersection, i.e. the bits that are both in `self` and `other`.
    fn intersection(&self, other: &Self) -> Self;

    /// Returns the union, i.e. the bits that are in `self` or `other`.
    fn union(&self, other: &Self) -> Self;

    /// Returns `true` if `self` and `other` have no bits in common. This is equivalent to checking for an empty
    /// intersection.
    fn is_disjoint(&self, other: &Self) -> bool {
        self.intersection(other).is_empty()
    }
}

macro_rules! impl_bitset {
    ($($t:ty)*) => ($(
        impl BitSet for $t {
            type Index = $t;

            #[inline]
            fn set(&mut self, index: $t) {
                *self |= 1 << index;
            }

            #[inline]
            fn clear(&mut self, index: $t) {
                *self &= !(1 << index);
            }

            #[inline]
            fn set_all(&mut self) {
                *self = !0;
            }

            #[inline]
            fn clear_all(&mut self) {
                *self = 0;
            }

            #[inline]
            fn get(&self, index: $t) -> bool {
                (*self & (1 << index)) != 0
            }

            #[inline]
            fn len(&self) -> $t {
                self.count_ones() as $t
            }

            #[inline]
            fn is_empty(&self) -> bool {
                *self == 0
            }

            #[inline]
            fn difference(&self, other: &$t) -> $t {
                self & !other
            }

            #[inline]
            fn symmetric_difference(&self, other: &$t) -> $t {
                self ^ other
            }

            #[inline]
            fn intersection(&self, other: &$t) -> $t {
                self & other
            }

            #[inline]
            fn union(&self, other: &$t) -> $t {
                self | other
            }

            #[inline]
            fn is_disjoint(&self, other: &$t) -> bool {
                self & other == 0
            }
        }
    )*)
}

impl_bitset!(u8 u16 u32 u64 usize);
impl_bitset!(i8 i16 i32 i64 isize);
