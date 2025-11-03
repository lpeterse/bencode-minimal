use super::Value;
use std::borrow::Cow;
use std::collections::BTreeMap;

pub struct Encoder<'a> {
    buf: &'a mut Vec<u8>,
}

impl<'a> Encoder<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        buf.clear();
        Self { buf }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn int(&mut self, n: i64) {
        self.raw_u8(b'i');
        if n < 0 {
            self.raw_u8(b'-');
        }
        self.raw_u64(n.unsigned_abs());
        self.raw_u8(b'e');
    }

    pub fn str(&mut self, s: &[u8]) {
        self.raw_usize(s.len());
        self.raw_u8(b':');
        self.raw_slice(s);
    }

    pub fn list(&mut self, l: &[Value<'_>]) {
        self.raw_u8(b'l');
        for v in l {
            self.value(v);
        }
        self.raw_u8(b'e');
    }

    pub fn dict(&mut self, d: &BTreeMap<Cow<'_, [u8]>, Value<'_>>) {
        self.raw_u8(b'd');
        for (k, v) in d {
            self.str(k);
            self.value(v);
        }
        self.raw_u8(b'e');
    }

    pub fn value(&mut self, v: &Value<'_>) {
        match v {
            Value::Int(i) => self.int(*i),
            Value::Str(s) => self.str(s),
            Value::List(l) => self.list(l),
            Value::Dict(d) => self.dict(d),
        }
    }

    pub fn raw_u8(&mut self, n: u8) {
        self.buf.push(n);
    }

    pub fn raw_u64(&mut self, n: u64) {
        let len = n.checked_ilog10().map(|i| i + 1).unwrap_or(1) as usize;
        let buf = self.alloc(len);
        let mut n = n;
        for b in buf.iter_mut().rev() {
            *b = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }

    pub fn raw_usize(&mut self, n: usize) {
        let len = n.checked_ilog10().map(|i| i + 1).unwrap_or(1) as usize;
        let buf = self.alloc(len);
        let mut n = n;
        for b in buf.iter_mut().rev() {
            *b = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }

    pub fn raw_slice(&mut self, data: &[u8]) {
        self.buf.extend(data);
    }

    pub fn alloc(&mut self, len: usize) -> &mut [u8] {
        let start = self.buf.len();
        self.buf.resize(start + len, 0);
        &mut self.buf[start..start + len]
    }
}
