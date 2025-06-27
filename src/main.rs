use nu_plugin::{
    EngineInterface, EvaluatedCall, MsgPackSerializer, Plugin, PluginCommand, SimplePluginCommand,
    serve_plugin,
};
use nu_protocol::{Category, Example, LabeledError, Signature, SyntaxShape, Type, Value};
use std::fs;
use tera::Tera;

mod helpers;
use crate::helpers::{unwrap_value_key, value_to_serde_json, wrap_top_level_if_needed};

#[cfg(test)]
mod tests;

/// Nushell plugin for rendering Tera templates with structured data.
pub struct TeraPlugin;

impl Plugin for TeraPlugin {
    /// Returns the plugin version from Cargo.toml.
    fn version(&self) -> String {
        // This automatically uses the version of your package from Cargo.toml as the plugin version
        // sent to Nushell
        env!("CARGO_PKG_VERSION").into()
    }

    /// Returns the list of commands provided by this plugin.
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            // Commands should be added here
            Box::new(Render),
        ]
    }
}

/// The main render command for the Tera plugin.
pub struct Render;

impl SimplePluginCommand for Render {
    type Plugin = TeraPlugin;

    /// The name of the command as used in Nushell.
    fn name(&self) -> &str {
        "tera-render"
    }

    /// The Nushell signature for the command, describing its parameters and usage.
    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::Any, Type::String)
            .required("template", SyntaxShape::Filepath, "Ruta al archivo .tera")
            // .switch("shout", "(FIXME) Yell it instead", None)
            .optional(
                "context",
                SyntaxShape::Any,
                "Datos de contexto (record o JSON path)",
            )
            .category(Category::Experimental)
    }

    /// A short description of the command for Nushell help.
    fn description(&self) -> &str {
        "(FIXME) help text for render"
    }

    /// Example usages of the command for Nushell help and testing.
    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "{ name: 'Akasha', projects: [ {'name': 'TheProject' , 'status': 'active' }]  } | tera-render example/template.tera",
            description: "Render template.tera with a record as context from the pipeline.\n\n\
                    template.tera:\n\
                    Hello, {{ name }}!Projects:\n\
                    {% for project in projects -%}\n\
                    - {{ project.name }} ({{ project.status }})\n\
                    {% endfor %}\n\n\
                    Other options:\n\
                    open data.json | wrap value | tera-render template.tera\n\
                    open data.json | tera-render template.tera\n\
                    ",
            result: Some(Value::test_string(
                "Hello, Akasha!\nProjects:\n- TheProject (active)\n\n",
            )),
        }]
    }

    /// The main entry point for the command. Handles reading the template, context, and rendering.
    fn run(
        &self,
        _plugin: &TeraPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let template_path: String = call.req(0)?;
        let context_arg: Option<Value> = call.opt(1)?;
        // if call.has_flag("shout")? {

        // Read template
        let template_content = fs::read_to_string(&template_path)
            .map_err(|e| LabeledError::new("Read error").with_label(e.to_string(), call.head))?;

        // Get data context (input pipeline or argument)
        let context = match context_arg {
            Some(val) => {
                if let Value::String { val: ref s, .. } = val {
                    if s.ends_with(".json") {
                        let file_content = std::fs::read_to_string(s).map_err(|e| {
                            LabeledError::new("Failed to read JSON file")
                                .with_label(e.to_string(), val.span())
                        })?;
                        let json: serde_json::Value =
                            serde_json::from_str(&file_content).map_err(|e| {
                                LabeledError::new("Failed to parse JSON file")
                                    .with_label(e.to_string(), val.span())
                            })?;
                        let context_json = unwrap_value_key(wrap_top_level_if_needed(json));
                        // println!("DEBUG context: {}", context_json);
                        let mut tera = Tera::default();
                        tera.add_raw_template(&template_path, &template_content)
                            .map_err(|e| {
                                LabeledError::new("Template error")
                                    .with_label(e.to_string(), call.head)
                            })?;
                        let context = tera::Context::from_serialize(context_json).map_err(|e| {
                            LabeledError::new("Tera context error")
                                .with_label(e.to_string(), val.span())
                        })?;
                        let output = tera.render(&template_path, &context).map_err(|e| {
                            LabeledError::new("Render error").with_label(e.to_string(), call.head)
                        })?;
                        return Ok(Value::string(output, call.head));
                    } else if s.ends_with(".yaml")
                        || s.ends_with(".yml")
                        || s.ends_with(".toml")
                        || s.ends_with(".csv")
                        || std::path::Path::new(s).exists()
                    {
                        return Err(LabeledError::new("Context is a file path, not data")
                            .with_label(
                                format!("You passed a file path ('{}') as context. Use 'open' to read the file: open {} | tera-render ...", s, s),
                                val.span()
                        ));
                    }
                }
                // Default context handling if not a file path string
                let context_json =
                    unwrap_value_key(wrap_top_level_if_needed(value_to_serde_json(val.clone())?));
                // println!("DEBUG context: {}", context_json);
                tera::Context::from_serialize(context_json).map_err(|e| {
                    LabeledError::new("Tera context error").with_label(e.to_string(), val.span())
                })?
            }
            None => {
                let context_json = unwrap_value_key(wrap_top_level_if_needed(value_to_serde_json(
                    input.clone(),
                )?));
                //println!("DEBUG context: {}", context_json);
                tera::Context::from_serialize(context_json).map_err(|e| {
                    LabeledError::new("Tera context error").with_label(e.to_string(), input.span())
                })?
            }
        };

        // Render with Tera
        let mut tera = Tera::default();
        tera.add_raw_template(&template_path, &template_content)
            .map_err(|e| {
                LabeledError::new("Template error").with_label(e.to_string(), call.head)
            })?;

        let output = tera
            .render(&template_path, &context)
            .map_err(|e| LabeledError::new("Render error").with_label(e.to_string(), call.head))?;

        Ok(Value::string(output, call.head))
    }
}

/// Entry point for the plugin binary.
fn main() {
    serve_plugin(&TeraPlugin, MsgPackSerializer);
}
