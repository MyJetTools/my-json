//! The trait exists so that my-http-utils can generate BOTH halves of a nested `#[http_body]`
//! object from one set of metadata, instead of writing it with `JsonValueWriter` and reading it
//! with serde. This test hand-writes exactly what such a derive macro would emit - using only the
//! crate's public API - so that if the trait ever stops being usable for codegen (most likely by
//! way of a lifetime that can not be named from a generated impl), it fails here rather than in
//! the downstream crate.

// A generated reader for a nested object and its inner object.

use my_json::json_reader::{JsonFirstLineIterator, JsonParseError, JsonValueReader, JsonValueRef};

#[derive(Debug, PartialEq)]
struct Inner {
    id: u64,
}

#[derive(Debug, PartialEq)]
struct S {
    a: u64,
    b: Option<String>,
    c: Vec<i32>,
    d: Inner,
}

impl<'s> JsonValueReader<'s> for Inner {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        // Can a downstream impl get the inner slice with lifetime 's using only public API?
        let inner: &'s [u8] = value.as_slice();
        let it = JsonFirstLineIterator::new(inner);
        let mut id = None;
        while let Some(kv) = it.get_next() {
            let (k, v) = kv?;
            let v = JsonValueRef::new(v.data.clone(), inner);
            if k.as_str()?.as_str() == "id" {
                id = Some(u64::from_json_value(&v)?);
            }
        }
        Ok(Inner {
            id: match id {
                Some(v) => v,
                None => u64::from_absent_json_value("id")?,
            },
        })
    }
}

impl<'s> JsonValueReader<'s> for S {
    fn from_json_value(value: &JsonValueRef<'s>) -> Result<Self, JsonParseError> {
        let inner: &'s [u8] = value.as_slice();
        let it = JsonFirstLineIterator::new(inner);
        let (mut a, mut b, mut c, mut d) = (None, None, None, None);
        while let Some(kv) = it.get_next() {
            let (k, v) = kv?;
            let v = JsonValueRef::new(v.data.clone(), inner);
            match k.as_str()?.as_str() {
                "a" => a = Some(u64::from_json_value(&v)?),
                "b" => b = Some(Option::<String>::from_json_value(&v)?),
                "c" => c = Some(Vec::<i32>::from_json_value(&v)?),
                "d" => d = Some(Inner::from_json_value(&v)?),
                _ => {}
            }
        }
        Ok(S {
            a: match a {
                Some(v) => v,
                None => u64::from_absent_json_value("a")?,
            },
            b: match b {
                Some(v) => v,
                None => Option::<String>::from_absent_json_value("b")?,
            },
            c: match c {
                Some(v) => v,
                None => Vec::<i32>::from_absent_json_value("c")?,
            },
            d: match d {
                Some(v) => v,
                None => Inner::from_absent_json_value("d")?,
            },
        })
    }
}

fn read<'s, T: JsonValueReader<'s>>(json: &'s str) -> Result<T, JsonParseError> {
    let v = my_json::j_path::get_value(json.as_bytes(), "v")
        .unwrap()
        .unwrap();
    T::from_json_value(&v)
}

#[test]
fn generated_struct_reader_composes() {
    // all fields present
    let r: S = read(r#"{"v":{"a":1,"b":"x","c":[1,2],"d":{"id":9}}}"#).unwrap();
    assert_eq!(
        r,
        S {
            a: 1,
            b: Some("x".into()),
            c: vec![1, 2],
            d: Inner { id: 9 }
        }
    );
    println!("all present -> OK");

    // b omitted entirely (what write_if_some does) -> None
    let r: S = read(r#"{"v":{"a":1,"c":[],"d":{"id":9}}}"#).unwrap();
    assert_eq!(r.b, None);
    println!("b omitted -> {:?}", r.b);

    // b explicit null -> None (must agree with omission)
    let r: S = read(r#"{"v":{"a":1,"b":null,"c":[],"d":{"id":9}}}"#).unwrap();
    assert_eq!(r.b, None);
    println!("b null -> {:?}", r.b);

    // required field missing -> clear error, not a default
    let e = read::<S>(r#"{"v":{"b":"x","c":[],"d":{"id":9}}}"#).unwrap_err();
    println!("a missing -> {}", e.to_string());
    assert!(e.to_string().contains("missing"));

    // Vec<S> of generated structs
    let r: Vec<S> =
        read(r#"{"v":[{"a":1,"c":[],"d":{"id":1}},{"a":2,"c":[3],"d":{"id":2}}]}"#).unwrap();
    assert_eq!(r.len(), 2);
    println!("Vec<S> -> OK ({} items)", r.len());
}
