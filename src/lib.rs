mod decoder;
mod encoder;
mod into_str;
mod try_from_value;
mod value;

pub use into_str::IntoStr;
pub use try_from_value::TryFromValue;
pub use value::{Dict, Int, List, Str, Value};

/// Create a [Value::Int] from [i64]
///
/// ```rust
/// use bencode_minimal::*;
///
/// let v = int!(42);
/// assert_eq!(v, Value::Int(42));
/// ```
#[macro_export]
macro_rules! int {
    ($x:expr) => {
        bencode_minimal::Value::Int($x)
    };
}

/// Create a [Value::Str] from all things [IntoStr] ([&str], [String], &[[u8]], [Vec]<[u8]> etc.)`
///
/// Values will be borrowed or owned depending on the input type and pass by value/reference.
///
/// ```rust
/// use bencode_minimal::*;
/// use std::borrow::Cow;
///
/// let v = str!("hello");
/// assert_eq!(v, Value::Str(Cow::Borrowed(b"hello")));
///
/// let v = str!("hello".to_string());
/// assert_eq!(v, Value::Str(Cow::Owned(b"hello".to_vec())));
///
/// let v = str!(b"hello");
/// assert_eq!(v, Value::Str(Cow::Borrowed(b"hello")));
///
/// let v = str!(b"world".to_vec());
/// assert_eq!(v, Value::Str(Cow::Owned(b"world".to_vec())));
/// ```
#[macro_export]
macro_rules! str {
    ($x:expr) => {
        bencode_minimal::Value::Str(bencode_minimal::IntoStr::into_str($x))
    };
}

/// Create a [Value::List] from a list of [Value]s
///
/// ```rust
/// use bencode_minimal::*;
/// use std::borrow::Cow;
///
/// let v = list![
///     int!(42),
///     str!("hello"),
/// ];
/// assert_eq!(v, Value::List(vec![Value::Int(42), Value::Str(Cow::Borrowed(b"hello"))]));
/// ```
#[macro_export]
macro_rules! list {
    ($($x:expr),* $(,)?) => {
        bencode_minimal::Value::List(vec![$($x),*])
    }
}

/// Create a [Value::Dict] from key-value pairs (keys like [str!], values as [Value]s)
///
/// ```rust
/// use bencode_minimal::*;
/// use std::borrow::Cow;
/// let v = dict! {
///     "age" => int!(42),
///     "name" => str!("John"),
/// };
/// assert_eq!(v, Value::Dict([
///     (Cow::Borrowed(b"age".as_ref()), Value::Int(42)),
///     (Cow::Borrowed(b"name".as_ref()), Value::Str(Cow::Borrowed(b"John"))),
/// ].into_iter().collect()));
/// ```
#[macro_export]
macro_rules! dict {
    ($($k:expr => $v:expr),* $(,)?) => {
        bencode_minimal::Value::Dict([$((bencode_minimal::IntoStr::into_str($k), $v)),*].into_iter().collect())
    };
    () => {
        bencode_minimal::Value::Dict(std::collections::BTreeMap::new())
    };
}
