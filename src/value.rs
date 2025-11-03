use super::decoder::Decoder;
use super::encoder::Encoder;
use super::TryFromValue;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// An alias for [i64]
pub type Int = i64;

/// An alias for byte string ([Cow] of `[u8]`)
pub type Str<'a> = Cow<'a, [u8]>;

/// An alias for a list of [Value]s
pub type List<'a> = Vec<Value<'a>>;

/// An alias for a dictionary mapping byte strings to [Value]s
pub type Dict<'a> = BTreeMap<Cow<'a, [u8]>, Value<'a>>;

/// A Bencode value is either an [Int], a [Str], a [List] or a [Dict]
///
/// Note that [Value] carries a lifetime parameter for borrowed data. This is useful for
/// two reasons:
///
/// 1. When encoding data, you can avoid unnecessary allocations by just referencing existing data.
///    Usually, the object is serialized right away and the object is dropped afterwards.
/// 2. When decoding data, all byte strings and dictionary keys are borrowed from the input buffer.
///    This avoids unnecessary allocations and is key to performance. Especially since keys are
///    often quite short, a separate [Vec] allocation per key would be very wasteful. The idea here is
///    also that after decoding, the object is used and dispatched right away and then dropped.
///    Actually required and useful data can be cloned out of the object if needed.
///
/// [Value]s are meant to be constructed using a combination of the provided macros [int!](super::int!),
/// [str!](super::str!), [list!](super::list!) and [dict!](super::dict!):
///
/// ```rust
/// use bencode_minimal::*;
///
/// let alice = "Alice".to_string();
///
/// let v = dict! {
///     "age" => int!(42),
///     "name" => str!("John"),                     // <-- static
///     "friends" => list![
///         str!(alice.as_str()),                   // <-- borrowed
///         dict! {
///             "name" => str!("Bob".to_string()),  // <-- owned
///             "data" => str!(vec![48u8, 49, 50]), // <-- owned
///         }
///     ]
/// };
///
/// let bin = v.encode();
/// assert_eq!(&bin, b"d3:agei42e7:friendsl5:Aliced4:data3:0124:name3:Bobee4:name4:Johne");
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value<'a> {
    Int(Int),
    Str(Str<'a>),
    List(List<'a>),
    Dict(Dict<'a>),
}

impl<'a> Value<'a> {
    /// Assume the value is a dictionary and get the value for the given key, converted into the desired type
    /// using [TryFromValue]
    ///
    /// Fails if the value is not a dictionary, the key does not exist or the value cannot be converted
    /// into the desired type.
    ///
    /// ```rust
    /// use bencode_minimal::*;
    ///
    /// let v = dict! {
    ///     "t" => str!(b"1234"),
    ///     "y" => str!(b"q"),
    ///     "q" => str!(b"ping"),
    ///     "a" => dict! {
    ///         "id" => str!(vec![5u8; 20]),
    ///     },
    /// };
    ///
    /// let ping_query = || {
    ///    let _ = v.get::<&str>("y").filter(|x| *x == "q")?;
    ///    let _ = v.get::<&str>("q").filter(|x| *x == "ping")?;
    ///    let a = v.get::<&Value>("a")?;
    ///    let t = v.get::<&[u8]>("t")?;
    ///    let id = a.get::<[u8;20]>("id")?;
    ///    Some((t, id))
    /// };
    ///
    /// let (t, id) = ping_query().unwrap();
    /// assert_eq!(t, b"1234");
    /// assert_eq!(id, [5u8; 20]);
    /// ```
    pub fn get<'b, T: TryFromValue<'b>>(&'b self, key: &'static str) -> Option<T> {
        let x = self.try_into::<&'b Dict<'b>>()?;
        let x = x.get(key.as_bytes())?;
        x.try_into()
    }

    /// Try to convert the [Value] into the desired type using [TryFromValue]
    ///
    /// Fails if the value cannot be converted into the desired type.
    pub fn try_into<'b, T: TryFromValue<'b>>(&'b self) -> Option<T> {
        T::try_from(self)
    }

    /// Quick encoding into a [Vec]<[u8]>
    ///
    /// The returned vector is freshly allocated and has a capacity of 1500 bytes to
    /// avoid multiple reallocations for typical use cases. Its length is adjusted to the
    /// actual encoded size.
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(1500);
        let mut e = Encoder::new(&mut v);
        e.value(self);
        v
    }

    /// Encode into the provided buffer
    ///
    /// The provided buffer is cleared before encoding and has the length of the encoded data afterwards.
    /// Its capacity is increased as needed but never decreased. Make sure to reuse the buffer when possible
    /// to avoid unnecessary allocations.Also, make sure to provide a buffer with sufficient initial capacity
    /// to avoid multiple reallocations. When passing the same buffer multiple times, its capacity will grow
    /// to the maximum size needed.
    pub fn encode_into(&self, buf: &mut Vec<u8>) {
        let mut e = Encoder::new(buf);
        e.clear();
        e.value(self);
    }

    /// Try to decode a [Value] from the provided buffer
    ///
    /// The `max_allocs` parameter limits the number of allocations that may be performed during decoding.
    /// This is useful to avoid denial-of-service attacks by providing maliciously crafted input that would
    /// cause excessive memory allocations. Each list item and dictionary entry counts as one allocation.
    /// If the limit is exceeded, decoding fails and `None` is returned.
    ///
    /// The returned [Value] borrows all byte strings from the input buffer. The value can therefor not outlive
    /// the input buffer. Either deconstruct the value right away (recommended) or use [Self::into_owned].
    pub fn decode(buf: &'a [u8], max_allocs: usize) -> Option<Self> {
        Decoder::new(buf, max_allocs).take_value()
    }

    /// Convert the value into an owned version
    ///
    /// All borrowed byte strings are cloned into owned [Vec]<[u8]>s. Byte strings that are already owned
    /// are moved into the new value without cloning. All [Vec]s and [BTreeMap]s get unfortunately
    /// recreated since there is no way to recycle them.
    pub fn into_owned(self) -> Value<'static> {
        match self {
            Value::Int(i) => Value::Int(i),
            Value::Str(s) => Value::Str(Cow::Owned(s.into_owned())),
            Value::List(l) => Value::List(l.into_iter().map(Value::into_owned).collect()),
            Value::Dict(d) => {
                Value::Dict(d.into_iter().map(|(k, v)| (Cow::Owned(k.into_owned()), v.into_owned())).collect())
            }
        }
    }
}

impl std::fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Str(s) => match std::str::from_utf8(s) {
                Ok(s) => write!(f, "{:?}", s),
                Err(_) => {
                    for i in s.iter() {
                        write!(f, "{:02x}", i)?;
                    }
                    Ok(())
                }
            },
            Value::List(l) => f.debug_list().entries(l.iter()).finish(),
            Value::Dict(d) => f
                .debug_map()
                .entries(d.iter().map(|(k, v)| {
                    let k = match std::str::from_utf8(k) {
                        Ok(s) => s.to_string(),
                        Err(_) => format!("{:?}", k),
                    };
                    (k, v)
                }))
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_01() {
        let value = Value::Int(0);
        let encoded = value.encode();
        assert_eq!(&encoded, b"i0e");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_int_02() {
        let value = Value::Int(1);
        let encoded = value.encode();
        assert_eq!(&encoded, b"i1e");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_int_03() {
        let value = Value::Int(-1);
        let encoded = value.encode();
        assert_eq!(&encoded, b"i-1e");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_int_04() {
        let value = Value::Int(10);
        let encoded = value.encode();
        assert_eq!(&encoded, b"i10e");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_int_05() {
        let value = Value::Int(-10);
        let encoded = value.encode();
        assert_eq!(&encoded, b"i-10e");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_int_06() {
        let value = Value::Int(42);
        let encoded = value.encode();
        assert_eq!(&encoded, b"i42e");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_int_07() {
        let value = Value::Int(-42);
        let encoded = value.encode();
        assert_eq!(&encoded, b"i-42e");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_str_01() {
        let value = Value::Str(Cow::Borrowed(b""));
        let encoded = value.encode();
        assert_eq!(&encoded, b"0:");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_str_02() {
        let value = Value::Str(Cow::Borrowed(b":"));
        let encoded = value.encode();
        assert_eq!(&encoded, b"1::");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_str_03() {
        let value = Value::Str(Cow::Borrowed(b"hello"));
        let encoded = value.encode();
        assert_eq!(&encoded, b"5:hello");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_str_04() {
        let value = Value::Str(Cow::Borrowed(b"helloworld"));
        let encoded = value.encode();
        assert_eq!(&encoded, b"10:helloworld");
        let value_ = Value::decode(&encoded, 0);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_list_01() {
        let value = Value::List(vec![]);
        let encoded = value.encode();
        assert_eq!(&encoded, b"le");
        let value_ = Value::decode(&encoded, 10);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_list_02() {
        let value = Value::List(vec![Value::Int(42), Value::Str(Cow::Borrowed(b"hello"))]);
        let encoded = value.encode();
        assert_eq!(&encoded, b"li42e5:helloe");
        let value_ = Value::decode(&encoded, 10);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_dict_01() {
        let value = Value::Dict(BTreeMap::new());
        let encoded = value.encode();
        assert_eq!(&encoded, b"de");
        let value_ = Value::decode(&encoded, 10);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_dict_02() {
        let mut dict = BTreeMap::new();
        dict.insert(b"age".into(), Value::Int(42));
        dict.insert(b"name".into(), Value::Str(Cow::Borrowed(b"John")));
        let value = Value::Dict(dict);
        let encoded = value.encode();
        assert_eq!(&encoded, b"d3:agei42e4:name4:Johne");
        let value_ = Value::decode(&encoded, 10);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_dict_03_reversed_order() {
        let mut dict = BTreeMap::new();
        dict.insert(b"name".into(), Value::Str(Cow::Borrowed(b"John")));
        dict.insert(b"age".into(), Value::Int(42));
        let value = Value::Dict(dict);

        let encoded = b"d4:name4:John3:agei42ee";
        let value_ = Value::decode(encoded.as_ref(), 10);
        assert_eq!(value_, Some(value));
    }

    #[test]
    fn test_dict_04_duplicate_keys() {
        let encoded = b"d3:agei30e3:agei40ee";
        let value = Value::decode(encoded.as_ref(), 10);
        assert!(value.is_none());
    }

    #[test]
    fn test_max_alloc_int() {
        let encoded = b"i42e";
        let value = Value::decode(encoded.as_ref(), 0);
        assert!(value.is_some());
    }

    #[test]
    fn test_max_alloc_str() {
        let encoded = b"5:hello";
        let value_ = Value::decode(encoded.as_ref(), 0);
        assert!(value_.is_some());
    }

    #[test]
    fn test_max_alloc_list_empty() {
        let encoded = b"le";
        let value = Value::decode(encoded.as_ref(), 0);
        assert!(value.is_some());
    }

    #[test]
    fn test_max_alloc_list_one() {
        let encoded = b"li42ee";

        let value = Value::decode(encoded.as_ref(), 0);
        assert!(value.is_none());

        let value = Value::decode(encoded.as_ref(), 1);
        assert!(value.is_some());
    }

    #[test]
    fn test_max_alloc_list_two() {
        let encoded = b"li1ei2ee";

        let value = Value::decode(encoded.as_ref(), 1);
        assert!(value.is_none());

        let value = Value::decode(encoded.as_ref(), 2);
        assert!(value.is_some());
    }

    #[test]
    fn test_max_alloc_dict_empty() {
        let encoded = b"de";
        let value = Value::decode(encoded.as_ref(), 0);
        assert!(value.is_some());
    }

    #[test]
    fn test_max_alloc_dict_one() {
        let encoded = b"d3:agei42ee";

        let value = Value::decode(encoded.as_ref(), 0);
        assert!(value.is_none());

        let value = Value::decode(encoded.as_ref(), 1);
        assert!(value.is_some());
    }

    #[test]
    fn test_max_alloc_dict_two() {
        let encoded = b"d3:agei42e4:name4:Johne";

        let value = Value::decode(encoded.as_ref(), 1);
        assert!(value.is_none());

        let value = Value::decode(encoded.as_ref(), 2);
        assert!(value.is_some());
    }
}
