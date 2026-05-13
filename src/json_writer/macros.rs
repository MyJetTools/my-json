/// Builds a JSON writer from a literal-like syntax. The first bracket inside the
/// macro decides the shape:
///
/// - `{ "k" => v, ... }` &rarr; [`JsonObjectWriter`](crate::json_writer::JsonObjectWriter)
/// - `[ v, ... ]`        &rarr; [`JsonArrayWriter`](crate::json_writer::JsonArrayWriter)
///
/// Nested `{ ... }` and `[ ... ]` literals are supported directly as values —
/// no need to re-invoke `my_json!`.
///
/// The macro returns the writer (not a `String`), so call `.build()` to get the
/// final JSON.
///
/// # Examples
/// ```
/// use my_json::my_json;
///
/// // Object with nested object and array literals
/// let s = my_json!({
///     "user"   => { "id" => 1, "name" => "Alice" },
///     "tags"   => ["admin", "editor"],
///     "active" => true,
/// }).build();
/// assert_eq!(
///     s,
///     r#"{"user":{"id":1,"name":"Alice"},"tags":["admin","editor"],"active":true}"#
/// );
///
/// // Array of nested objects
/// let s = my_json!([
///     { "id" => 1 },
///     { "id" => 2 },
/// ]).build();
/// assert_eq!(s, r#"[{"id":1},{"id":2}]"#);
///
/// // Empty values
/// assert_eq!(my_json!({}).build(), "{}");
/// assert_eq!(my_json!([]).build(), "[]");
/// ```
#[macro_export]
macro_rules! my_json {
    ({ $($tt:tt)* }) => {{
        #[allow(unused_mut)]
        let mut __w = $crate::json_writer::JsonObjectWriter::new();
        $crate::__my_json_obj!(__w; $($tt)*);
        __w
    }};
    ([ $($tt:tt)* ]) => {{
        #[allow(unused_mut)]
        let mut __w = $crate::json_writer::JsonArrayWriter::new();
        $crate::__my_json_arr!(__w; $($tt)*);
        __w
    }};
}

/// Internal: object-body TT muncher. Not part of the public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __my_json_obj {
    // end
    ($w:ident;) => {};
    ($w:ident; ,) => {};

    // key => { nested object } [, rest]
    ($w:ident; $key:expr => { $($inner:tt)* } , $($rest:tt)*) => {
        $w = $w.write($key, $crate::my_json!({ $($inner)* }));
        $crate::__my_json_obj!($w; $($rest)*);
    };
    ($w:ident; $key:expr => { $($inner:tt)* }) => {
        $w = $w.write($key, $crate::my_json!({ $($inner)* }));
    };

    // key => [ nested array ] [, rest]
    ($w:ident; $key:expr => [ $($inner:tt)* ] , $($rest:tt)*) => {
        $w = $w.write($key, $crate::my_json!([ $($inner)* ]));
        $crate::__my_json_obj!($w; $($rest)*);
    };
    ($w:ident; $key:expr => [ $($inner:tt)* ]) => {
        $w = $w.write($key, $crate::my_json!([ $($inner)* ]));
    };

    // key => expr , rest
    ($w:ident; $key:expr => $value:expr , $($rest:tt)*) => {
        $w = $w.write($key, $value);
        $crate::__my_json_obj!($w; $($rest)*);
    };
    // key => expr (last)
    ($w:ident; $key:expr => $value:expr) => {
        $w = $w.write($key, $value);
    };
}

/// Internal: array-body TT muncher. Not part of the public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __my_json_arr {
    // end
    ($w:ident;) => {};
    ($w:ident; ,) => {};

    // { nested object } [, rest]
    ($w:ident; { $($inner:tt)* } , $($rest:tt)*) => {
        $w = $w.write($crate::my_json!({ $($inner)* }));
        $crate::__my_json_arr!($w; $($rest)*);
    };
    ($w:ident; { $($inner:tt)* }) => {
        $w = $w.write($crate::my_json!({ $($inner)* }));
    };

    // [ nested array ] [, rest]
    ($w:ident; [ $($inner:tt)* ] , $($rest:tt)*) => {
        $w = $w.write($crate::my_json!([ $($inner)* ]));
        $crate::__my_json_arr!($w; $($rest)*);
    };
    ($w:ident; [ $($inner:tt)* ]) => {
        $w = $w.write($crate::my_json!([ $($inner)* ]));
    };

    // expr , rest
    ($w:ident; $value:expr , $($rest:tt)*) => {
        $w = $w.write($value);
        $crate::__my_json_arr!($w; $($rest)*);
    };
    // expr (last)
    ($w:ident; $value:expr) => {
        $w = $w.write($value);
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn object_basic() {
        let s = crate::my_json!({ "fff" => "ffff" }).build();
        assert_eq!(s, r#"{"fff":"ffff"}"#);
    }

    #[test]
    fn object_multi_key() {
        let s = crate::my_json!({
            "a" => "x",
            "b" => 42,
            "c" => true,
        })
        .build();
        assert_eq!(s, r#"{"a":"x","b":42,"c":true}"#);
    }

    #[test]
    fn object_empty() {
        assert_eq!(crate::my_json!({}).build(), "{}");
    }

    #[test]
    fn array_basic() {
        let s = crate::my_json!([1, 2, 3]).build();
        assert_eq!(s, "[1,2,3]");
    }

    #[test]
    fn array_empty() {
        assert_eq!(crate::my_json!([]).build(), "[]");
    }

    // -- Nested literals --------------------------------------------------

    #[test]
    fn nested_object_literal_in_object() {
        let s = crate::my_json!({
            "outer" => { "inner" => "v" },
        })
        .build();
        assert_eq!(s, r#"{"outer":{"inner":"v"}}"#);
    }

    #[test]
    fn nested_array_literal_in_object() {
        let s = crate::my_json!({
            "name" => "Alice",
            "tags" => ["admin", "editor"],
            "nums" => [1, 2, 3],
        })
        .build();
        assert_eq!(
            s,
            r#"{"name":"Alice","tags":["admin","editor"],"nums":[1,2,3]}"#
        );
    }

    #[test]
    fn nested_object_literal_in_array() {
        let s = crate::my_json!([
            { "id" => 1, "name" => "Alice" },
            { "id" => 2, "name" => "Bob" },
        ])
        .build();
        assert_eq!(s, r#"[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]"#);
    }

    #[test]
    fn nested_array_literal_in_array() {
        let s = crate::my_json!([[1, 2], [3, 4], [5, 6]]).build();
        assert_eq!(s, "[[1,2],[3,4],[5,6]]");
    }

    #[test]
    fn deeply_nested_literals() {
        let s = crate::my_json!({
            "company" => "TechCorp",
            "departments" => [
                {
                    "name"     => "Engineering",
                    "head"     => { "id" => 1, "name" => "Alice" },
                    "members"  => [
                        { "id" => 1, "name" => "Alice" },
                        { "id" => 2, "name" => "Bob" },
                    ],
                },
                {
                    "name"    => "Marketing",
                    "head"    => { "id" => 3, "name" => "Carol" },
                    "members" => [{ "id" => 3, "name" => "Carol" }],
                },
            ],
        })
        .build();

        assert!(s.starts_with(r#"{"company":"TechCorp","departments":["#));
        assert!(s.contains(r#""head":{"id":1,"name":"Alice"}"#));
        assert!(s.contains(r#""members":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]"#));
        assert!(s.ends_with("]}"));
    }

    #[test]
    fn mixed_with_expressions() {
        let name = "Alice".to_string();
        let scores: Vec<u32> = vec![10, 20, 30];

        let s = crate::my_json!({
            "name"     => name.as_str(),
            "scores"   => scores,
            "computed" => 2 + 2,
            "meta"     => { "ok" => true },
        })
        .build();
        assert_eq!(
            s,
            r#"{"name":"Alice","scores":[10,20,30],"computed":4,"meta":{"ok":true}}"#
        );
    }

    #[test]
    fn optional_values_serialise_as_null() {
        let phone: Option<&str> = None;
        let email: Option<&str> = Some("a@b.com");

        let s = crate::my_json!({
            "phone" => phone,
            "email" => email,
        })
        .build();
        assert_eq!(s, r#"{"phone":null,"email":"a@b.com"}"#);
    }
}
