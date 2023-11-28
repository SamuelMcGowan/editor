use std::ops::{Add, Div, Mul, Sub};

macro_rules! vec2_type {
    ($name:ident $t:ty) => {
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            pub x: $t,
            pub y: $t,
        }

        impl $name {
            pub const ZERO: Self = Self::splat(0);

            #[inline]
            pub const fn new(x: $t, y: $t) -> Self {
                Self { x, y }
            }

            #[inline]
            pub const fn splat(n: $t) -> Self {
                Self::new(n, n)
            }

            #[inline]
            #[must_use]
            pub fn min(self, rhs: Self) -> Self {
                self.join(rhs, <$t>::min)
            }

            #[inline]
            #[must_use]
            pub fn max(self, rhs: Self) -> Self {
                self.join(rhs, <$t>::max)
            }

            #[inline]
            #[must_use]
            pub fn saturating_add(self, rhs: Self) -> Self {
                self.join(rhs, <$t>::saturating_add)
            }

            #[inline]
            #[must_use]
            pub fn saturating_sub(self, rhs: Self) -> Self {
                self.join(rhs, <$t>::saturating_sub)
            }

            #[inline]
            #[must_use]
            pub fn saturating_mul(self, rhs: Self) -> Self {
                self.join(rhs, <$t>::saturating_mul)
            }

            #[inline]
            #[must_use]
            pub fn checked_add(self, rhs: Self) -> Option<Self> {
                self.try_join(rhs, <$t>::checked_add)
            }

            #[inline]
            #[must_use]
            pub fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.try_join(rhs, <$t>::checked_sub)
            }

            #[inline]
            #[must_use]
            pub fn checked_mul(self, rhs: Self) -> Option<Self> {
                self.try_join(rhs, <$t>::checked_mul)
            }

            #[inline]
            #[must_use]
            pub fn checked_div(self, rhs: Self) -> Option<Self> {
                self.try_join(rhs, <$t>::checked_div)
            }

            #[inline]
            pub fn cmp_eq(&self, rhs: Self) -> Comparison {
                self.cmp(rhs, <$t>::eq)
            }

            #[inline]
            pub fn cmp_ne(&self, rhs: Self) -> Comparison {
                self.cmp(rhs, <$t>::ne)
            }

            #[inline]
            pub fn cmp_lt(&self, rhs: Self) -> Comparison {
                self.cmp(rhs, <$t>::lt)
            }

            #[inline]
            pub fn cmp_gt(&self, rhs: Self) -> Comparison {
                self.cmp(rhs, <$t>::gt)
            }

            #[inline]
            pub fn cmp_le(&self, rhs: Self) -> Comparison {
                self.cmp(rhs, <$t>::le)
            }

            #[inline]
            pub fn cmp_ge(&self, rhs: Self) -> Comparison {
                self.cmp(rhs, <$t>::ge)
            }

            #[inline]
            fn join(self, rhs: Self, f: impl Fn($t, $t) -> $t) -> Self {
                Self::new(f(self.x, rhs.x), f(self.y, rhs.y))
            }

            #[inline]
            fn join_t(self, rhs: $t, f: impl Fn($t, $t) -> $t) -> Self {
                Self::new(f(self.x, rhs), f(self.y, rhs))
            }

            #[inline]
            fn try_join(self, rhs: Self, f: impl Fn($t, $t) -> Option<$t>) -> Option<Self> {
                Some(Self::new(f(self.x, rhs.x)?, f(self.y, rhs.y)?))
            }

            #[inline]
            fn cmp(self, rhs: Self, f: impl Fn(&$t, &$t) -> bool) -> Comparison {
                Comparison {
                    x: f(&self.x, &rhs.x),
                    y: f(&self.y, &rhs.y),
                }
            }
        }

        impl From<[$t; 2]> for $name {
            #[inline]
            fn from(value: [$t; 2]) -> Self {
                Self::new(value[0], value[1])
            }
        }

        impl From<$name> for [$t; 2] {
            #[inline]
            fn from(value: $name) -> Self {
                [value.x, value.y]
            }
        }
    };
}

vec2_type! { OffsetU16 u16 }
vec2_type! { OffsetUsize usize }

macro_rules! impl_op_vec2 {
    ($vec:ty, $t:ty = $($trait:ident $f:ident),*) => {
        $(
            impl $trait<$vec> for $vec {
                type Output = Self;

                #[inline]
                fn $f(self, rhs: Self) -> Self::Output {
                    self.join(rhs, <$t>::$f)
                }
            }
        )*
    };
}

impl_op_vec2! { OffsetU16, u16 = Add add, Sub sub }
impl_op_vec2! { OffsetUsize, usize = Add add, Sub sub }

macro_rules! impl_op_t {
    ($vec:ty, $t:ty = $($trait:ident $f:ident),*) => {
        $(
            impl $trait<$t> for $vec {
                type Output = Self;

                #[inline]
                fn $f(self, rhs: $t) -> Self::Output {
                    self.join_t(rhs, <$t>::$f)
                }
            }
        )*
    };
}

impl_op_t! { OffsetU16, u16 = Add add, Sub sub, Mul mul, Div div }
impl_op_t! { OffsetUsize, usize = Add add, Sub sub, Mul mul, Div div }

impl OffsetU16 {
    #[inline]
    pub fn area(&self) -> usize {
        self.x as usize * self.y as usize
    }
}

macro_rules! conversions {
    ($($src:ty => $dest:ty),*) => {
        $(
            impl From<$src> for $dest {
                #[inline]
                fn from(value: $src) -> Self {
                    Self::new(value.x.into(), value.y.into())
                }
            }
        )*
    };
}

conversions! { OffsetU16 => OffsetUsize }

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Comparison {
    pub x: bool,
    pub y: bool,
}

impl Comparison {
    #[inline]
    pub fn both(&self) -> bool {
        self.x && self.y
    }

    #[inline]
    pub fn either(&self) -> bool {
        self.x || self.y
    }
}
