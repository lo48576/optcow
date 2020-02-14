//! Optional CoW.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::borrow::{Cow, ToOwned};

use core::{borrow::Borrow, cmp::Ordering, hash, mem};

/// An efficient alternative for `Option<Cow<'_, B>>`.
pub enum OptCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    /// `None`.
    None,
    /// Borrowed data.
    Borrowed(&'a B),
    /// Owned data.
    Owned(<B as ToOwned>::Owned),
}

impl<'a, B> OptCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    /// Returns `true` if the `OptCow` has some data.
    #[inline]
    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }

    /// Returns `true` if the `OptCow` has no data.
    #[inline]
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }

    /// Acquires a mutable reference to the owned form of the data.
    pub fn to_mut(&mut self) -> Option<&mut <B as ToOwned>::Owned> {
        match self {
            Self::None => None,
            Self::Borrowed(borrowed) => {
                *self = Self::Owned(borrowed.to_owned());
                match self {
                    Self::Owned(owned) => Some(owned),
                    _ => unreachable!("`OptCow::Owned` is substituted"),
                }
            }
            Self::Owned(owned) => Some(owned),
        }
    }

    /// Extracts the owned data.
    pub fn into_owned(self) -> Option<<B as ToOwned>::Owned> {
        match self {
            Self::None => None,
            Self::Borrowed(borrowed) => Some(borrowed.to_owned()),
            Self::Owned(owned) => Some(owned),
        }
    }

    /// Returns reference to the inner value.
    pub fn as_ref(&self) -> Option<&B> {
        match self {
            Self::None => None,
            Self::Borrowed(borrowed) => Some(borrowed),
            Self::Owned(owned) => Some(owned.borrow()),
        }
    }

    /// Converts the value into `Cow` if it is not `None`.
    #[inline]
    pub fn into_cow(self) -> Option<Cow<'a, B>> {
        self.into()
    }

    /// Takes the value out of the `OptCow`, leaving a `None` in its place.
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }

    /// Replaces `self` with the given value, and returns the old replaced value.
    #[inline]
    pub fn replace(&mut self, v: Cow<'a, B>) -> Self {
        mem::replace(self, v.into())
    }
}

impl<B> Default for OptCow<'_, B>
where
    B: ToOwned + ?Sized,
{
    fn default() -> Self {
        Self::None
    }
}

impl<B> Clone for OptCow<'_, B>
where
    B: ToOwned + ?Sized,
{
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Borrowed(borrowed) => Self::Borrowed(borrowed),
            Self::Owned(owned) => {
                let borrowed: &B = owned.borrow();
                Self::Owned(borrowed.to_owned())
            }
        }
    }
}

impl<'b, 'c, B, C> PartialEq<OptCow<'c, C>> for OptCow<'b, B>
where
    B: PartialEq<C> + ToOwned + ?Sized,
    C: ToOwned + ?Sized,
{
    fn eq(&self, other: &OptCow<'c, C>) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (Some(lhs), Some(rhs)) => PartialEq::eq(lhs, rhs),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<B> Eq for OptCow<'_, B> where B: Eq + ToOwned + ?Sized {}

impl<B, C> PartialOrd<OptCow<'_, C>> for OptCow<'_, B>
where
    B: PartialOrd<C> + ToOwned + ?Sized,
    C: ToOwned + ?Sized,
{
    fn partial_cmp(&self, other: &OptCow<'_, C>) -> Option<Ordering> {
        match (self.as_ref(), other.as_ref()) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (Some(lhs), Some(rhs)) => PartialOrd::partial_cmp(lhs, rhs),
        }
    }
}

impl<B> Ord for OptCow<'_, B>
where
    B: Ord + ToOwned + ?Sized,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.as_ref(), other.as_ref()) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(lhs), Some(rhs)) => Ord::cmp(lhs, rhs),
        }
    }
}

impl<B> hash::Hash for OptCow<'_, B>
where
    B: hash::Hash + ToOwned + ?Sized,
{
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.as_ref(), state)
    }
}

impl<'a, B> From<&'a B> for OptCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    fn from(v: &'a B) -> Self {
        Self::Borrowed(v)
    }
}

impl<'a, B> From<Cow<'a, B>> for OptCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    fn from(v: Cow<'a, B>) -> Self {
        match v {
            Cow::Borrowed(borrowed) => Self::Borrowed(borrowed),
            Cow::Owned(owned) => Self::Owned(owned),
        }
    }
}

impl<'a, B> From<Option<Cow<'a, B>>> for OptCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    fn from(v: Option<Cow<'a, B>>) -> Self {
        match v {
            None => Self::None,
            Some(Cow::Borrowed(borrowed)) => Self::Borrowed(borrowed),
            Some(Cow::Owned(owned)) => Self::Owned(owned),
        }
    }
}

impl<'a, B> From<OptCow<'a, B>> for Option<Cow<'a, B>>
where
    B: 'a + ToOwned + ?Sized,
{
    fn from(v: OptCow<'a, B>) -> Self {
        match v {
            OptCow::None => None,
            OptCow::Borrowed(borrowed) => Some(Cow::Borrowed(borrowed)),
            OptCow::Owned(owned) => Some(Cow::Owned(owned)),
        }
    }
}
