use crate::json_writer::JsonValueWriter;

pub enum WriteJsonValue<'s, T: JsonValueWriter> {
    Value(T),
    ValueRef(&'s T),
    Vec(Vec<T>),
    Slice(&'s [T]),
}

impl<'s, T: JsonValueWriter> From<T> for WriteJsonValue<'s, T> {
    fn from(value: T) -> Self {
        WriteJsonValue::Value(value)
    }
}

impl<'s, T: JsonValueWriter> From<&'s T> for WriteJsonValue<'s, T> {
    fn from(value: &'s T) -> Self {
        WriteJsonValue::ValueRef(value)
    }
}

impl<'s, T: JsonValueWriter> From<Vec<T>> for WriteJsonValue<'s, T> {
    fn from(value: Vec<T>) -> Self {
        WriteJsonValue::Vec(value)
    }
}

impl<'s, T: JsonValueWriter> From<&'s [T]> for WriteJsonValue<'s, T> {
    fn from(value: &'s [T]) -> Self {
        WriteJsonValue::Slice(value)
    }
}

impl<'s, T: JsonValueWriter> From<&'s Vec<T>> for WriteJsonValue<'s, T> {
    fn from(value: &'s Vec<T>) -> Self {
        WriteJsonValue::Slice(value.as_slice())
    }
}
