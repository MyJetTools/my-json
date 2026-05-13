# My JSON - Rust JSON Processing Library

A high-performance Rust library for JSON processing with a powerful JSON path query system and an ergonomic JSON writer.

## Features

- **JSON Path queries** for easy data extraction (`user.name`, `items[0].id`, `users[].name`)
- **Zero-copy reads** over byte slices
- **Path-based updates** via `j_update`
- **Fluent writer API** with `JsonObjectWriter` / `JsonArrayWriter`
- **Declarative macro** `my_json!` for terse construction (bracket-dispatched: `{}` builds an object, `[]` builds an array)
- **Async streaming** of JSON arrays and JSON-L
- Optional `rust_decimal` support via the `decimal` feature

## Installation

```toml
[dependencies]
my-json = { git = "https://github.com/MyJetTools/my-json.git" }
```

Or, when published to a registry, pin the version:

```toml
[dependencies]
my-json = "0.3.2"
```

Optional decimal support:

```toml
[dependencies]
my-json = { version = "0.3.2", features = ["decimal"] }
```

## Quick Start — Reading

```rust
use my_json::j_path::get_value;

let json = r#"{"name": "John", "age": 30, "city": "New York"}"#;

let name = get_value(json.as_bytes(), "name").unwrap().unwrap();
assert_eq!(name.as_str().unwrap().as_str(), "John");

let city = get_value(json.as_bytes(), "city").unwrap().unwrap();
assert_eq!(city.as_str().unwrap().as_str(), "New York");
```

## Quick Start — Writing

```rust
use my_json::my_json;

let json = my_json!({
    "name"      => "John Doe",
    "age"       => 30,
    "is_active" => true,
}).build();

assert_eq!(json, r#"{"name":"John Doe","age":30,"is_active":true}"#);
```

---

## JSON Path Queries

The path API lives in the `my_json::j_path` module. Two main entry points:

| Function | Returns | Purpose |
|---|---|---|
| `get_value(json, path)` | `Result<Option<JsonValueRef>, _>` | Single value at a path |
| `get_value_as_vec(json, path)` | `Result<Vec<JsonValueRef>, _>` | All matches (supports `[]` array fan-out) |

### Basic Object Access

```rust
use my_json::j_path::get_value;

let json = r#"{"user": {"name": "Alice", "email": "alice@example.com"}}"#;

let name = get_value(json.as_bytes(), "user.name").unwrap().unwrap();
assert_eq!(name.as_str().unwrap().as_str(), "Alice");

let email = get_value(json.as_bytes(), "user.email").unwrap().unwrap();
assert_eq!(email.as_str().unwrap().as_str(), "alice@example.com");
```

### Array Element Access by Index

```rust
use my_json::j_path::get_value;

let json = r#"{"items": [{"id": 1, "name": "Item 1"}, {"id": 2, "name": "Item 2"}]}"#;

let first = get_value(json.as_bytes(), "items[0]").unwrap().unwrap();
assert!(first.is_object());

let name1 = get_value(json.as_bytes(), "items[0].name").unwrap().unwrap();
assert_eq!(name1.as_str().unwrap().as_str(), "Item 1");

let name2 = get_value(json.as_bytes(), "items[1].name").unwrap().unwrap();
assert_eq!(name2.as_str().unwrap().as_str(), "Item 2");
```

### Fanning Out Over Every Element — `[]`

`get_value_as_vec` supports a trailing `[]` to collect every element of an array:

```rust
use my_json::j_path::get_value_as_vec;

let json = r#"{
    "users": [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"},
        {"id": 3, "name": "Charlie"}
    ]
}"#;

// Every user object
let users = get_value_as_vec(json.as_bytes(), "users[]").unwrap();
assert_eq!(users.len(), 3);

// Every name string
let names = get_value_as_vec(json.as_bytes(), "users[].name").unwrap();
assert_eq!(names.len(), 3);
assert_eq!(names[0].as_str().unwrap().as_str(), "Alice");
assert_eq!(names[1].as_str().unwrap().as_str(), "Bob");
assert_eq!(names[2].as_str().unwrap().as_str(), "Charlie");
```

### Complex Nested Structures

```rust
use my_json::j_path::get_value;

let json = r#"{
    "company": {
        "name": "TechCorp",
        "departments": [
            {
                "name": "Engineering",
                "employees": [
                    {"id": 1, "name": "John", "role": "Developer"},
                    {"id": 2, "name": "Jane", "role": "Senior Developer"}
                ]
            },
            {
                "name": "Marketing",
                "employees": [{"id": 3, "name": "Bob", "role": "Manager"}]
            }
        ]
    }
}"#;

let company_name = get_value(json.as_bytes(), "company.name").unwrap().unwrap();
assert_eq!(company_name.as_str().unwrap().as_str(), "TechCorp");

let dept = get_value(json.as_bytes(), "company.departments[0].name").unwrap().unwrap();
assert_eq!(dept.as_str().unwrap().as_str(), "Engineering");

let role = get_value(
    json.as_bytes(),
    "company.departments[0].employees[0].role",
).unwrap().unwrap();
assert_eq!(role.as_str().unwrap().as_str(), "Developer");
```

### Error Handling

```rust
use my_json::j_path::get_value;

let json = r#"{"user": {"name": "Alice"}}"#;

match get_value(json.as_bytes(), "user.age") {
    Ok(None)         => println!("Path not found"),
    Ok(Some(value))  => println!("Found: {:?}", value),
    Err(e)           => println!("Parse error: {:?}", e),
}

// Invalid JSON
let bad = r#"{"user": {"name": "Alice"}"#; // missing closing brace
if let Err(e) = get_value(bad.as_bytes(), "user.name") {
    println!("JSON parsing error: {:?}", e);
}
```

### Working with Value Types

`JsonValueRef` exposes type-checking and extraction methods:

```rust
use my_json::j_path::get_value;

let json = r#"{
    "string":     "hello",
    "number":     42,
    "boolean":    true,
    "null_value": null,
    "array":      [1, 2, 3],
    "object":     {"key": "value"}
}"#;

let v = get_value(json.as_bytes(), "string").unwrap().unwrap();
assert!(v.is_string());
assert_eq!(v.as_str().unwrap().as_str(), "hello");

let v = get_value(json.as_bytes(), "number").unwrap().unwrap();
assert!(v.is_number());
assert_eq!(v.unwrap_as_number().unwrap(), Some(42));

let v = get_value(json.as_bytes(), "boolean").unwrap().unwrap();
assert!(v.is_bool());
assert_eq!(v.unwrap_as_bool(), Some(true));

let v = get_value(json.as_bytes(), "null_value").unwrap().unwrap();
assert!(v.is_null());

let v = get_value(json.as_bytes(), "array").unwrap().unwrap();
assert!(v.is_array());

let v = get_value(json.as_bytes(), "object").unwrap().unwrap();
assert!(v.is_object());
```

### Querying Further from a `JsonValueRef`

A `JsonValueRef` can itself be queried:

```rust
use my_json::j_path::get_value;

let json = r#"{"company": {"name": "TechCorp", "ceo": {"name": "Alice"}}}"#;
let company = get_value(json.as_bytes(), "company").unwrap().unwrap();

let ceo_name = company.j_query_as_value("ceo.name").unwrap().unwrap();
assert_eq!(ceo_name.as_str().unwrap().as_str(), "Alice");
```

### Updating a Value at a Path — `j_update`

```rust
use my_json::j_path::j_update;

let json = r#"{"name":"John","age":30}"#;
let updated = j_update(json, "age", 31).unwrap();
assert_eq!(updated, r#"{"name":"John","age":31}"#);
```

`j_update` accepts any value that implements `JsonValueWriter` (strings, numbers, bool, nested writers, …).

---

## JSON Writing

Two writer types and two declarative macros.

### Declarative Macro — `my_json!`

A single macro picks the writer based on the first bracket inside:

| Form                        | Builds              |
|-----------------------------|---------------------|
| `my_json!({ k => v, ... })` | `JsonObjectWriter`  |
| `my_json!([ v, v, ... ])`   | `JsonArrayWriter`   |

The macro returns the writer — call `.build()` for the final `String`. **Nested `{ ... }` and `[ ... ]` literals are accepted directly as values** at any depth.

#### 1. The simplest object

```rust
use my_json::my_json;

let s = my_json!({ "fff" => "ffff" }).build();
assert_eq!(s, r#"{"fff":"ffff"}"#);
```

#### 2. Object with mixed value types

```rust
use my_json::my_json;

let s = my_json!({
    "name"    => "Alice",
    "age"     => 30_u32,
    "score"   => 4.5_f64,
    "active"  => true,
    "tagline" => "He said \"hi\"",   // escaping is automatic
}).build();
assert_eq!(
    s,
    r#"{"name":"Alice","age":30,"score":4.5,"active":true,"tagline":"He said \"hi\""}"#
);
```

#### 3. A plain array

```rust
use my_json::my_json;

let s = my_json!([1, 2, 3]).build();
assert_eq!(s, "[1,2,3]");

let s = my_json!(["red", "green", "blue"]).build();
assert_eq!(s, r#"["red","green","blue"]"#);
```

#### 4. Empty object / empty array

```rust
use my_json::my_json;

assert_eq!(my_json!({}).build(), "{}");
assert_eq!(my_json!([]).build(), "[]");
```

#### 5. Nested object literal inside an object

Write the inner `{ ... }` directly — no `my_json!` re-invocation needed:

```rust
use my_json::my_json;

let s = my_json!({
    "user" => { "id" => 1, "name" => "Alice" },
    "ok"   => true,
}).build();
assert_eq!(s, r#"{"user":{"id":1,"name":"Alice"},"ok":true}"#);
```

#### 6. Nested array literal inside an object

```rust
use my_json::my_json;

let s = my_json!({
    "name" => "Alice",
    "tags" => ["admin", "editor"],
    "nums" => [1, 2, 3],
}).build();
assert_eq!(
    s,
    r#"{"name":"Alice","tags":["admin","editor"],"nums":[1,2,3]}"#
);
```

#### 7. Array of objects (literal)

```rust
use my_json::my_json;

let s = my_json!([
    { "id" => 1, "name" => "Alice" },
    { "id" => 2, "name" => "Bob"   },
]).build();
assert_eq!(s, r#"[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]"#);
```

#### 8. Array of arrays

```rust
use my_json::my_json;

let s = my_json!([[1, 2], [3, 4], [5, 6]]).build();
assert_eq!(s, "[[1,2],[3,4],[5,6]]");
```

#### 9. Deeply nested structures

Mix `{ ... }` and `[ ... ]` literals freely at any depth:

```rust
use my_json::my_json;

let s = my_json!({
    "company" => "TechCorp",
    "departments" => [
        {
            "name"    => "Engineering",
            "head"    => { "id" => 1, "name" => "Alice" },
            "members" => [
                { "id" => 1, "name" => "Alice" },
                { "id" => 2, "name" => "Bob"   },
            ],
        },
        {
            "name"    => "Marketing",
            "head"    => { "id" => 3, "name" => "Carol" },
            "members" => [{ "id" => 3, "name" => "Carol" }],
        },
    ],
}).build();
```

#### 10. Mixing literals with computed values

Any expression of a `JsonValueWriter` type is accepted on the right side of `=>` — strings, numbers, `Vec<T>`, `Option<T>`, etc.:

```rust
use my_json::my_json;

let name = "Alice".to_string();
let scores: Vec<u32> = vec![10, 20, 30];

let s = my_json!({
    "name"     => name.as_str(),
    "scores"   => scores,             // Vec<T> renders as an array
    "computed" => 2 + 2,
    "meta"     => { "ok" => true },   // nested literal
}).build();
assert_eq!(
    s,
    r#"{"name":"Alice","scores":[10,20,30],"computed":4,"meta":{"ok":true}}"#
);
```

#### 11. Optional fields (`Option<T>`)

`Option<T>` writes `null` when `None`:

```rust
use my_json::my_json;

let middle_name: Option<&str> = None;
let phone: Option<&str>       = Some("+1-555");

let s = my_json!({
    "first_name"  => "Alice",
    "middle_name" => middle_name,   // serialised as null
    "phone"       => phone,
}).build();
assert_eq!(
    s,
    r#"{"first_name":"Alice","middle_name":null,"phone":"+1-555"}"#
);
```

> If you'd rather **skip** the key entirely when `None`, use the fluent
> `JsonObjectWriter::write_if_some(key, opt)` method instead — see the next section.

#### 12. A real-world shape — API response

```rust
use my_json::my_json;

let payload = my_json!({
    "ok"    => true,
    "count" => 2_u32,
    "items" => [
        { "id" => 1, "title" => "Item 1" },
        { "id" => 2, "title" => "Item 2" },
    ],
    "paging" => {
        "page"     => 1_u32,
        "per_page" => 50_u32,
        "next"     => Option::<&str>::None,
    },
}).build();

assert!(payload.contains(r#""count":2"#));
assert!(payload.contains(r#""title":"Item 1""#));
assert!(payload.contains(r#""next":null"#));
```

### `JsonObjectWriter` — Fluent API

```rust
use my_json::json_writer::JsonObjectWriter;

let json = JsonObjectWriter::new()
    .write("name",      "John Doe")
    .write("age",       30)
    .write("is_active", true)
    .build();

assert_eq!(json, r#"{"name":"John Doe","age":30,"is_active":true}"#);
```

#### Nested Objects

```rust
use my_json::json_writer::JsonObjectWriter;

let json = JsonObjectWriter::new()
    .write("user", "john_doe")
    .write_json_object("profile", |profile| {
        profile
            .write("first_name", "John")
            .write("last_name",  "Doe")
            .write("email",      "john@example.com")
    })
    .write_json_object("settings", |settings| {
        settings
            .write("theme",         "dark")
            .write("notifications", true)
    })
    .build();
```

#### Embedding Arrays

```rust
use my_json::json_writer::JsonObjectWriter;

let json = JsonObjectWriter::new()
    .write("name", "Project Alpha")
    .write_json_array("tags", |tags| {
        tags.write("rust").write("json").write("performance")
    })
    .write_json_array("members", |members| {
        members
            .write_json_object(|m| m.write("id", 1).write("name", "Alice"))
            .write_json_object(|m| m.write("id", 2).write("name", "Bob"))
    })
    .build();
```

#### Conditional Writes

```rust
use my_json::json_writer::JsonObjectWriter;

let include_age   = true;
let maybe_email: Option<&str> = Some("a@b.com");
let maybe_phone: Option<&str> = None;

let json = JsonObjectWriter::new()
    .write("name", "Alice")
    .write_if("age", 30, include_age)               // include only when condition is true
    .write_if_some("email", maybe_email)            // include only when Some
    .write_if_some("phone", maybe_phone)            // skipped — None
    .write_json_object_if("admin", false, |o| {     // skipped — flag is false
        o.write("level", 5)
    })
    .build();

assert_eq!(json, r#"{"name":"Alice","age":30,"email":"a@b.com"}"#);
```

#### Writing an Iterator as an Array Value

```rust
use my_json::json_writer::JsonObjectWriter;

let scores = vec![98, 75, 82];

let json = JsonObjectWriter::new()
    .write("user", "Alice")
    .write_iter("scores", scores.iter().copied())
    .build();

assert_eq!(json, r#"{"user":"Alice","scores":[98,75,82]}"#);
```

### `JsonArrayWriter` — Fluent API

```rust
use my_json::json_writer::JsonArrayWriter;

let json = JsonArrayWriter::new()
    .write("apple")
    .write("banana")
    .write("cherry")
    .build();

assert_eq!(json, r#"["apple","banana","cherry"]"#);
```

#### Mixed Types

```rust
use my_json::json_writer::JsonArrayWriter;

let json = JsonArrayWriter::new()
    .write("string_value")
    .write(42)
    .write(true)
    .write_json_object(|obj| obj.write("key", "value"))
    .build();

assert_eq!(json, r#"["string_value",42,true,{"key":"value"}]"#);
```

#### Empty Array as a Value

```rust
use my_json::json_writer::{JsonObjectWriter, EmptyJsonArray};

let json = JsonObjectWriter::new()
    .write("array", EmptyJsonArray)
    .build();

assert_eq!(json, r#"{"array":[]}"#);
```

### Complex Nested Structure

```rust
use my_json::json_writer::JsonObjectWriter;

let json = JsonObjectWriter::new()
    .write("company", "TechCorp")
    .write_json_array("departments", |depts| {
        depts
            .write_json_object(|dept| {
                dept
                    .write("name",   "Engineering")
                    .write("budget", 1_000_000)
                    .write_json_array("employees", |emps| {
                        emps
                            .write_json_object(|e| {
                                e.write("id", 1)
                                 .write("name", "Alice")
                                 .write("role", "Senior Developer")
                                 .write_json_array("skills", |s| {
                                     s.write("rust").write("python").write("docker")
                                 })
                            })
                            .write_json_object(|e| {
                                e.write("id", 2)
                                 .write("name", "Bob")
                                 .write("role", "Developer")
                                 .write_json_array("skills", |s| {
                                     s.write("javascript").write("react")
                                 })
                            })
                    })
            })
            .write_json_object(|dept| {
                dept
                    .write("name",   "Marketing")
                    .write("budget", 500_000)
                    .write_json_array("employees", |emps| {
                        emps.write_json_object(|e| {
                            e.write("id", 3)
                             .write("name", "Carol")
                             .write("role", "Marketing Manager")
                        })
                    })
            })
    })
    .build();
```

### Output Methods

`build` consumes the writer. The non-consuming `build_into` / `write_into_vec` methods are useful when you want to append to an existing buffer or keep the writer alive:

```rust
use my_json::json_writer::JsonObjectWriter;

// 1) build into a new String (consumes the writer)
let json: String = JsonObjectWriter::new()
    .write("k", "v")
    .build();

// 2) append to an existing String
let writer = JsonObjectWriter::new().write("k", "v");
let mut buf = String::from("prefix=");
writer.build_into(&mut buf);
// buf == "prefix={\"k\":\"v\"}"

// 3) append to a Vec<u8>
let writer = JsonObjectWriter::new().write("k", "v");
let mut bytes = Vec::new();
writer.write_into_vec(&mut bytes);
```

### Supported Value Types

Anything implementing `JsonValueWriter` can be written:

- Integers: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `usize`, `isize`
- Floats: `f32`, `f64`
- `bool`
- Strings: `String`, `&str`
- `JsonNullValue` (writes `null`)
- `EmptyJsonArray` (writes `[]`)
- Nested writers: `JsonObjectWriter`, `JsonArrayWriter`
- `Option<T>` of any of the above (writes `null` when `None`)
- `rust_decimal::Decimal` — when the `decimal` feature is enabled

---

## API Reference

### `j_path` Module

```rust
pub fn get_value<'s, 'd>(
    json: &'s [u8],
    path: impl Into<rust_extensions::StrOrString<'d>>,
) -> Result<Option<JsonValueRef<'s>>, JsonParseError>;

pub fn get_value_as_vec<'s, 'd>(
    json: &'s [u8],
    path: impl Into<rust_extensions::StrOrString<'d>>,
) -> Result<Vec<JsonValueRef<'s>>, JsonParseError>;

pub fn j_update<'s, 'd>(
    json: &'s str,
    path: impl Into<rust_extensions::StrOrString<'d>>,
    value_to_replace: impl JsonValueWriter,
) -> Result<String, JsonParseError>;
```

### Path Syntax

| Pattern               | Meaning                                  |
|-----------------------|------------------------------------------|
| `key`                 | Direct property                          |
| `parent.child`        | Nested property                          |
| `array[0]`            | Indexed array element                    |
| `users[0].profile.name` | Combined nesting and indexing          |
| `users[]`             | Every element of the array (use `get_value_as_vec`) |
| `users[].name`        | `name` of every element                  |

### `JsonValueRef` Methods

| Method | Purpose |
|---|---|
| `is_string()` / `as_str()` | String value (returns `StrOrString`) |
| `as_raw_str()` / `as_unescaped_str()` | Raw / unescaped string slice |
| `is_number()` / `unwrap_as_number()` | Integer value |
| `is_double()` / `unwrap_as_double()` | Floating-point value |
| `is_bool()` / `unwrap_as_bool()` | Boolean value |
| `is_null()` | Null check |
| `is_array()` / `unwrap_as_array()` | Array iterator |
| `is_object()` / `unwrap_as_object()` | Object iterator |
| `as_date_time()` | Parse value as `DateTimeAsMicroseconds` |
| `as_slice()` / `as_bytes()` | Raw JSON bytes for the value |
| `j_query_as_value(path)` | Re-query into this value |
| `j_query_as_vec(path)` | Re-query into this value, collect matches |

## Performance Notes

- **Zero-copy reads**: `JsonValueRef` borrows from the input byte slice.
- **Lazy parsing**: paths are resolved on demand, not by pre-parsing the entire document.
- **Streaming**: `array_parser_async` and `json_l_iterator(_async)` support large inputs.

## Error Handling

`JsonParseError` covers malformed JSON, invalid path syntax, type mismatches, and out-of-bounds access.

## Contributing

Pull requests are welcome.

## License

See the `LICENSE.md` file.
