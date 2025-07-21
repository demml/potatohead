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
        let _ = MyOutput {
            value: 42,
            message: "Hello, world!".to_string(),
        };

        let schema = ::schemars::schema_for!(MyOutput);

        let output = MyOutput::get_structured_output_schema();
        assert_eq!(output, schema);
    }
}
