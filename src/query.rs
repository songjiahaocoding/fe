use serde_json::Value;

use crate::path::{JsonPath, PathSegment};

pub fn get<'value>(value: &'value Value, path: &JsonPath) -> Vec<&'value Value> {
    let mut matches = vec![value];

    for segment in path.segments() {
        let mut next = Vec::new();
        for candidate in matches {
            match segment {
                PathSegment::Key(key) => {
                    if let Value::Object(object) = candidate
                        && let Some(value) = object.get(key)
                    {
                        next.push(value);
                    }
                }
                PathSegment::Index(index) => {
                    if let Value::Array(array) = candidate
                        && let Some(index) = resolve_existing_index(*index, array.len())
                    {
                        next.push(&array[index]);
                    }
                }
                PathSegment::Wildcard => match candidate {
                    Value::Array(array) => next.extend(array.iter()),
                    Value::Object(object) => next.extend(object.values()),
                    _ => {}
                },
            }
        }
        matches = next;
    }

    matches
}

pub fn exists(value: &Value, path: &JsonPath) -> bool {
    !get(value, path).is_empty()
}

fn resolve_existing_index(index: i64, len: usize) -> Option<usize> {
    let index = if index < 0 { len as i64 + index } else { index };
    if index < 0 || index >= len as i64 {
        None
    } else {
        Some(index as usize)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{path::JsonPath, query};

    #[test]
    fn gets_wildcard_values() {
        let value = json!({ "items": [{ "name": "a" }, { "name": "b" }] });
        let path = "$.items[*].name".parse::<JsonPath>().unwrap();
        let matches = query::get(&value, &path);
        assert_eq!(matches, vec![&json!("a"), &json!("b")]);
    }
}
