use potato_type::StructuredOutput;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct MyOutput {
    pub value: i32,
    pub message: String,
}

impl StructuredOutput for MyOutput {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_output_serialization() {
        let output = MyOutput {
            value: 42,
            message: "Hello, world!".to_string(),
        };

        let schema = ::schemars::schema_for!(MyOutput);
        let assert_schema = serde_json::json!({
            "type": "json_schema",
            "json_schema": {
                "name": MyOutput::type_name(),
                "schema": schema,
                "strict": true
            }
        });

        let output = MyOutput::get_structured_output_schema();
        assert_eq!(output, assert_schema);
    }
}
