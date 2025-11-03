use super::{Dict, List, Value};

/// Conversion from [Value]
pub trait TryFromValue<'a>: Sized {
    fn try_from(value: &'a Value) -> Option<Self>;
}

macro_rules! from {
    ($t:ident, $value:ident as $item:ident => $expr:expr) => {{
        if let Value::$t($item) = $value {
            $expr
        } else {
            None
        }
    }};
}

impl<'a> TryFromValue<'a> for i64 {
    fn try_from(value: &'a Value<'a>) -> Option<Self> {
        from!(Int, value as v => Some(*v))
    }
}

impl<'a> TryFromValue<'a> for &'a [u8] {
    fn try_from(value: &'a Value) -> Option<Self> {
        from!(Str, value as v => Some(v.as_ref()))
    }
}

impl<'a, const N: usize> TryFromValue<'a> for [u8; N] {
    fn try_from(value: &'a Value) -> Option<Self> {
        from!(Str, value as v => TryFrom::try_from(v.as_ref()).ok())
    }
}

impl<'a, A: TryFromValue<'a>, B: TryFromValue<'a>> TryFromValue<'a> for (A, B) {
    fn try_from(value: &'a Value) -> Option<Self> {
        from!(List, value as v => {
            let a = v.get(0).map(A::try_from)?;
            let b = v.get(1).map(B::try_from)?;
            a.zip(b)
        })
    }
}

impl<'a> TryFromValue<'a> for &'a str {
    fn try_from(value: &'a Value) -> Option<Self> {
        from!(Str, value as v => std::str::from_utf8(v).ok())
    }
}

impl<'a> TryFromValue<'a> for &'a List<'a> {
    fn try_from(value: &'a Value) -> Option<Self> {
        from!(List, value as v => Some(v))
    }
}

impl<'a> TryFromValue<'a> for &'a Dict<'a> {
    fn try_from(value: &'a Value) -> Option<Self> {
        from!(Dict, value as v => Some(v))
    }
}

impl<'a> TryFromValue<'a> for &'a Value<'a> {
    fn try_from(value: &'a Value) -> Option<Self> {
        Some(value)
    }
}
