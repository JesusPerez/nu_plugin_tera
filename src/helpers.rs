use nu_protocol::{LabeledError, Value};

/// Convert a Nushell Value to a serde_json::Value suitable for Tera context.
///
/// - Records are converted to JSON objects.
/// - Lists, strings, ints, and bools are wrapped in an object with key "value".
/// - Other types return an error.
pub fn value_to_serde_json(value: Value) -> Result<serde_json::Value, LabeledError> {
    match value {
        Value::Record { val, .. } => {
            let record = &*val;
            let mut map = serde_json::Map::new();
            for (col, val) in record.columns().zip(record.values()) {
                map.insert(col.clone(), value_to_serde_json(val.clone())?);
            }
            Ok(serde_json::Value::Object(map))
        }
        Value::List { vals, .. } => {
            let vec = vals
                .into_iter()
                .map(value_to_serde_json)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(serde_json::Value::Array(vec))
        }
        Value::String { val, .. } => Ok(serde_json::Value::String(val)),
        Value::Int { val, .. } => Ok(serde_json::Value::Number(val.into())),
        Value::Bool { val, .. } => Ok(serde_json::Value::Bool(val)),
        _ => Err(LabeledError::new("Type not supported")
            .with_label("Use records, lists or primitives", value.span())),
    }
}

/// Removes the top-level 'value' key if it is the only key in the object, and always returns an object (wraps non-objects as { "value": ... }).
pub fn unwrap_value_key(json: serde_json::Value) -> serde_json::Value {
    let unwrapped = if let serde_json::Value::Object(mut map) = json {
        if map.len() == 1 {
            if let Some(inner) = map.remove("value") {
                return unwrap_value_key(inner);
            }
        }
        serde_json::Value::Object(map)
    } else {
        json
    };
    match unwrapped {
        serde_json::Value::Object(_) => unwrapped,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), other);
            serde_json::Value::Object(map)
        }
    }
}

/// Wraps the top-level value if it is not an object.
pub fn wrap_top_level_if_needed(json: serde_json::Value) -> serde_json::Value {
    match json {
        serde_json::Value::Object(_) => json,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), other);
            serde_json::Value::Object(map)
        }
    }
}
