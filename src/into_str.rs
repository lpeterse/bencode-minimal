use super::Str;
use std::borrow::Cow;

/// Conversion into [Str]
pub trait IntoStr<'a> {
    fn into_str(self) -> Str<'a>;
}

impl<'a, 'b: 'a> IntoStr<'a> for &'b [u8] {
    fn into_str(self) -> Str<'a> {
        Cow::Borrowed(self)
    }
}

impl<'a, 'b: 'a, const N: usize> IntoStr<'a> for &'b [u8; N] {
    fn into_str(self) -> Str<'a> {
        Cow::Borrowed(self)
    }
}

impl<'a, 'b: 'a> IntoStr<'a> for &'b str {
    fn into_str(self) -> Str<'a> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<'a> IntoStr<'a> for Vec<u8> {
    fn into_str(self) -> Str<'a> {
        Cow::Owned(self)
    }
}

impl<'a, const N: usize> IntoStr<'a> for [u8; N] {
    fn into_str(self) -> Str<'a> {
        Cow::Owned(self.to_vec())
    }
}

impl<'a> IntoStr<'a> for String {
    fn into_str(self) -> Str<'a> {
        Cow::Owned(self.into_bytes())
    }
}
