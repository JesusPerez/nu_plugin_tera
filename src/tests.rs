// use super::*;
use crate::helpers::{unwrap_value_key, value_to_serde_json, wrap_top_level_if_needed};
use crate::{Render, TeraPlugin};
use nu_protocol::{Record, Span, Value};
use tera::Tera;

/// Runs the plugin test examples using nu_plugin_test_support.
#[test]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    use nu_plugin_test_support::PluginTest;

    // This will automatically run the examples specified in your command and compare their actual
    // output against what was specified in the example. You can remove this test if the examples
    // can't be tested this way, but we recommend including it if possible.

    PluginTest::new("tera", TeraPlugin.into())?.test_command_examples(&Render)
}
#[test]
fn test_value_to_serde_json_record() {
    let record = Record::from_raw_cols_vals(
        vec!["name".to_string(), "age".to_string()],
        vec![
            Value::string("Akasha", Span::test_data()),
            Value::int(42, Span::test_data()),
        ],
        Span::test_data(),
        Span::test_data(),
    )
    .expect("failed to create test record");
    let val = Value::record(record, Span::test_data());
    let json = value_to_serde_json(val).unwrap();
    assert_eq!(json["name"], "Akasha");
    assert_eq!(json["age"], 42);
}

#[test]
fn test_value_to_serde_json_list() {
    let val = Value::list(
        vec![
            Value::int(1, Span::test_data()),
            Value::int(2, Span::test_data()),
        ],
        Span::test_data(),
    );
    let json = value_to_serde_json(val).unwrap();
    assert_eq!(json, serde_json::json!([1, 2]));
}

#[test]
fn test_value_to_serde_json_string() {
    let val = Value::string("hello", Span::test_data());
    let json = value_to_serde_json(val).unwrap();
    assert_eq!(json, serde_json::json!("hello"));
}

#[test]
fn test_unwrap_value_key_simple() {
    let json = serde_json::json!({"value": {"name": "Akasha"}});
    let unwrapped = unwrap_value_key(json);
    assert_eq!(unwrapped["name"], "Akasha");
}

#[test]
fn test_unwrap_value_key_nested() {
    let json = serde_json::json!({"value": {"value": {"name": "Akasha"}}});
    let unwrapped = unwrap_value_key(json);
    assert_eq!(unwrapped["name"], "Akasha");
}

#[test]
fn test_unwrap_value_key_non_object() {
    let json = serde_json::json!(42);
    let unwrapped = unwrap_value_key(json);
    assert_eq!(unwrapped["value"], 42);
}

#[test]
fn test_unwrap_value_key_object() {
    let json = serde_json::json!({"name": "Akasha"});
    let unwrapped = unwrap_value_key(json);
    assert_eq!(unwrapped["name"], "Akasha");
}

#[test]
fn test_render_pipeline() {
    let template = "Hello, {{ name }}!";
    let mut tera = Tera::default();
    tera.add_raw_template("test", template).unwrap();
    let record = Record::from_raw_cols_vals(
        vec!["name".to_string()],
        vec![Value::string("Akasha", Span::test_data())],
        Span::test_data(),
        Span::test_data(),
    )
    .expect("failed to create test record");
    let val = Value::record(record, Span::test_data());
    let context_json =
        unwrap_value_key(wrap_top_level_if_needed(value_to_serde_json(val).unwrap()));
    let context = tera::Context::from_serialize(context_json).unwrap();
    let output = tera.render("test", &context).unwrap();
    assert_eq!(output, "Hello, Akasha!");
}
