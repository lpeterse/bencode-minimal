# bencode-minimal

A Bencode library depending only on the Rust standard library.

## Features

- No dependencies except Rust standard library
- MIT license
- Less than 600 LOC
- Protection against malicious user input (limiting the allocations per decoding)
- Borrowing from the input buffer for reduced allocations
- Convencience methods and macros for interpreting Bencode's byte strings as UTF-8 strings

## Usage

```rust
use bencode_minimal::*;

let v1 = dict! {
    "name" => str!("John"),
    "age" => int!(42),
    "friends" => list![
        str!("Alice"),
        dict! {
            "name" => str!("Bob"),
            "data" => str!(vec![48u8, 49, 50]),
        }
    ]
};

let bin = v1.encode();
assert_eq!(&bin, b"d3:agei42e7:friendsl5:Aliced4:data3:0124:name3:Bobee4:name4:Johne");

let v2 = Value::decode(&bin, 10).unwrap();
println!("{:#?}", v2);
```

Output:

```rust
{
    "age": 42,
    "friends": [
        "Alice",
        {
            "data": "012",
            "name": "Bob",
        },
    ],
    "name": "John",
}
```

## Noteworthy

### Strict output, relaxed input

Dictionaries are always encoded with keys in ascending order as the format mandates.
When it comes to decoding, any order is accepted. This avoids headache when dealing with
non-compliant peers.

Duplicate keys are forbidden for security reasons, though.

### Encoding is a total function

Every instance of a Bencode value can be encoded. No errors to handle in this case.

### Decoding returns `Optional`

The decoder makes no attempt to report any details about failures or the failure location.
This is for simplicity: This is a decoder for a strict and simple format and not a parser.
In most circumstances this information would not be looked at anyway. If you need this
information for debugging, use something else.
