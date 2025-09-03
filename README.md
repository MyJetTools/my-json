# My JSON - Rust JSON Processing Library

A high-performance Rust library for JSON processing with a powerful JSON path querying system.

## Features

- **Fast JSON parsing** with zero-copy operations
- **JSON Path queries** for easy data extraction
- **Memory efficient** - works with byte slices
- **Async support** for large files
- **Flexible iteration** over JSON structures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
my-json = "0.3.1"
```

## Quick Start

```rust
use my_json::j_path;

fn main() {
    let json_data = r#"{"name": "John", "age": 30, "city": "New York"}"#;
    
    // Extract a simple value
    let name = j_path(json_data.as_bytes(), "name").unwrap().unwrap();
    println!("Name: {}", name.as_str().unwrap().as_str());
    
    // Extract nested values
    let city = j_path(json_data.as_bytes(), "city").unwrap().unwrap();
    println!("City: {}", city.as_str().unwrap().as_str());
}
```

## JSON Path Examples

### Basic Object Access

```rust
use my_json::j_path;

let json = r#"{"user": {"name": "Alice", "email": "alice@example.com"}}"#;

// Access nested properties
let name = j_path(json.as_bytes(), "user.name").unwrap().unwrap();
assert_eq!(name.as_str().unwrap().as_str(), "Alice");

let email = j_path(json.as_bytes(), "user.email").unwrap().unwrap();
assert_eq!(email.as_str().unwrap().as_str(), "alice@example.com");
```

### Array Access

```rust
use my_json::j_path;

let json = r#"{"items": [{"id": 1, "name": "Item 1"}, {"id": 2, "name": "Item 2"}]}"#;

// Access array elements by index
let first_item = j_path(json.as_bytes(), "items[0]").unwrap().unwrap();
assert!(first_item.is_object());

let first_item_name = j_path(json.as_bytes(), "items[0].name").unwrap().unwrap();
assert_eq!(first_item_name.as_str().unwrap().as_str(), "Item 1");

let second_item_name = j_path(json.as_bytes(), "items[1].name").unwrap().unwrap();
assert_eq!(second_item_name.as_str().unwrap().as_str(), "Item 2");
```

### Complex Nested Structures

```rust
use my_json::j_path;

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
                "employees": [
                    {"id": 3, "name": "Bob", "role": "Manager"}
                ]
            }
        ]
    }
}"#;

// Navigate complex nested structures
let company_name = j_path(json.as_bytes(), "company.name").unwrap().unwrap();
assert_eq!(company_name.as_str().unwrap().as_str(), "TechCorp");

let first_dept_name = j_path(json.as_bytes(), "company.departments[0].name").unwrap().unwrap();
assert_eq!(first_dept_name.as_str().unwrap().as_str(), "Engineering");

let first_employee = j_path(json.as_bytes(), "company.departments[0].employees[0]").unwrap().unwrap();
assert!(first_employee.is_object());

let employee_role = j_path(json.as_bytes(), "company.departments[0].employees[0].role").unwrap().unwrap();
assert_eq!(employee_role.as_str().unwrap().as_str(), "Developer");
```

### Error Handling

```rust
use my_json::j_path;

let json = r#"{"user": {"name": "Alice"}}"#;

// Handle missing paths gracefully
let result = j_path(json.as_bytes(), "user.age");
match result {
    Ok(None) => println!("Path 'user.age' not found"),
    Ok(Some(value)) => println!("Found value: {:?}", value),
    Err(e) => println!("Error: {:?}", e),
}

// Handle invalid JSON
let invalid_json = r#"{"user": {"name": "Alice"}"#; // Missing closing brace
let result = j_path(invalid_json.as_bytes(), "user.name");
if let Err(e) = result {
    println!("JSON parsing error: {:?}", e);
}
```

### Working with Different Data Types

```rust
use my_json::j_path;

let json = r#"{
    "string": "hello",
    "number": 42,
    "boolean": true,
    "null_value": null,
    "array": [1, 2, 3],
    "object": {"key": "value"}
}"#;

// String values
let string_val = j_path(json.as_bytes(), "string").unwrap().unwrap();
assert_eq!(string_val.as_str().unwrap().as_str(), "hello");

// Numeric values
let number_val = j_path(json.as_bytes(), "number").unwrap().unwrap();
assert!(number_val.is_number());

// Boolean values
let bool_val = j_path(json.as_bytes(), "boolean").unwrap().unwrap();
assert!(bool_val.is_boolean());

// Null values
let null_val = j_path(json.as_bytes(), "null_value").unwrap().unwrap();
assert!(null_val.is_null());

// Array values
let array_val = j_path(json.as_bytes(), "array").unwrap().unwrap();
assert!(array_val.is_array());

// Object values
let object_val = j_path(json.as_bytes(), "object").unwrap().unwrap();
assert!(object_val.is_object());
```

## API Reference

### Main Function

```rust
pub fn j_path<'s>(json: &'s [u8], path: &str) -> Result<Option<JsonValueRef<'s>>, JsonParseError>
```

- **`json`**: Byte slice containing JSON data
- **`path`**: JSON path string (e.g., "user.name", "items[0].id")
- **Returns**: `Result<Option<JsonValueRef>>` where:
  - `Ok(Some(value))` - Path found, value extracted
  - `Ok(None)` - Path not found
  - `Err(e)` - JSON parsing error

### JSON Path Syntax

- **Simple property**: `"key"`
- **Nested property**: `"parent.child"`
- **Array access**: `"array[0]"`
- **Combined**: `"users[0].profile.name"`

### Value Types

The `JsonValueRef` provides methods to check and extract values:

- `is_string()` / `as_str()` - String values
- `is_number()` / `as_number()` - Numeric values  
- `is_boolean()` / `as_boolean()` - Boolean values
- `is_null()` - Null values
- `is_array()` - Array values
- `is_object()` - Object values

## Performance Notes

- **Zero-copy**: The library works with byte slices and doesn't allocate new memory for extracted values
- **Lazy parsing**: JSON is parsed only as needed during path traversal
- **Efficient iteration**: Uses optimized iterators for arrays and objects

## Error Handling

The library provides detailed error information through `JsonParseError`:

- **Invalid JSON syntax**
- **Malformed paths**
- **Type mismatches**
- **Index out of bounds**

## Examples Directory

For more complex examples, check the `examples/` directory in the repository.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the terms specified in the LICENSE file.
