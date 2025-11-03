use super::Value;
use std::borrow::Cow;
use std::collections::BTreeMap;

pub struct Decoder<'a> {
    buf: &'a [u8],
    rem_allocs: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8], max_allocs: usize) -> Self {
        Self { buf, rem_allocs: max_allocs }
    }

    pub fn take_int(&mut self) -> Option<i64> {
        self.take_u8_eq(b'i')?;
        let i = self.take_i64()?;
        self.take_u8_eq(b'e')?;
        Some(i)
    }

    pub fn take_list(&mut self) -> Option<Vec<Value<'a>>> {
        self.take_u8_eq(b'l')?;
        let mut list = Vec::new();
        while self.buf.get(0)? != &b'e' {
            self.alloc(1)?;
            list.push(self.take_value()?);
        }
        self.take_u8_eq(b'e')?;
        Some(list)
    }

    pub fn take_str(&mut self) -> Option<Cow<'a, [u8]>> {
        let len = self.take_usize()?;
        self.take_u8_eq(b':')?;
        self.take_u8_slice(len).map(Cow::Borrowed)
    }

    pub fn take_dict(&mut self) -> Option<BTreeMap<Cow<'a, [u8]>, Value<'a>>> {
        self.take_u8_eq(b'd')?;
        let mut dict = BTreeMap::new();
        while let Some(key) = self.take_str() {
            self.alloc(1)?;
            let value = self.take_value()?;
            if dict.insert(key, value).is_some() {
                return None; // Duplicate keys are forbidden
            }
        }
        self.take_u8_eq(b'e')?;
        Some(dict)
    }

    pub fn take_value(&mut self) -> Option<Value<'a>> {
        match self.buf.get(0)? {
            b'i' => self.take_int().map(Value::Int),
            b'l' => self.take_list().map(Value::List),
            b'd' => self.take_dict().map(Value::Dict),
            b'0'..=b'9' => self.take_str().map(Value::Str),
            _ => None,
        }
    }

    pub fn take_u8_eq(&mut self, c: u8) -> Option<()> {
        let (_, t) = self.buf.split_first().filter(|x| x.0 == &c)?;
        self.buf = t;
        Some(())
    }

    pub fn take_u8_if(&mut self, f: impl FnOnce(&u8) -> bool) -> Option<u8> {
        let (h, t) = self.buf.split_first().filter(|x| f(x.0))?;
        self.buf = t;
        Some(*h)
    }

    pub fn take_u8_slice(&mut self, n: usize) -> Option<&'a [u8]> {
        let (h, t) = self.buf.split_at_checked(n)?;
        self.buf = t;
        Some(h)
    }

    pub fn take_i64(&mut self) -> Option<i64> {
        let s = self.take_u8_eq(b'-');
        let mut r: i64 = (self.take_u8_if(u8::is_ascii_digit)? - b'0').into();
        while let Some(x) = self.take_u8_if(u8::is_ascii_digit) {
            r = r.checked_mul(10)?;
            r = r.checked_add((x - b'0').into())?;
        }
        s.map(|_| -r).or(Some(r))
    }

    pub fn take_usize(&mut self) -> Option<usize> {
        let mut r: usize = (self.take_u8_if(u8::is_ascii_digit)? - b'0').into();
        while let Some(x) = self.take_u8_if(u8::is_ascii_digit) {
            r = r.checked_mul(10)?;
            r = r.checked_add((x - b'0').into())?;
        }
        Some(r)
    }

    fn alloc(&mut self, n: usize) -> Option<()> {
        self.rem_allocs = self.rem_allocs.checked_sub(n)?;
        Some(())
    }
}
