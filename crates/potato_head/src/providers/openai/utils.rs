use crate::common::pyobject_to_json;
use crate::error::PotatoHeadError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict};
use serde_json::{json, Value};

fn ensure_strict_json_schema<'py>(schema: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    if !schema.is_instance_of::<PyDict>() {
        return Err(PotatoHeadError::new_err("Schema is not a dictionary"));
    }

    let schema_type = schema.getattr("type")?.extract::<String>()?;

    if schema_type != "object" && !schema.hasattr("additionalProperties")? {
        schema.setattr("additionalProperties", false)?;
    }

    Ok(schema)
}

pub fn convert_pydantic_to_openai_json_schema(
    py: Python,
    model: &Bound<'_, PyAny>,
) -> PyResult<Value> {
    // Import pydantic and builtins once
    let pydantic = py.import("pydantic")?;
    let builtins = py.import("builtins")?;
    let issubclass = builtins.getattr("issubclass")?;

    // Get pydantic classes
    let basemodel = pydantic.getattr("BaseModel")?;

    // Check if it's a BaseModel subclass
    if !issubclass
        .call1((model, basemodel))?
        .downcast::<PyBool>()?
        .is_true()
    {
        return Err(PotatoHeadError::new_err(
            "Model is not a subclass of pydantic BaseModel",
        ));
    }

    // Get model name once
    let name = model.getattr("__name__")?.extract::<String>()?;
    let schema = model.call_method0("model_json_schema")?;

    // Create JSON response
    Ok(json!({
        "type": "json_schema",
        "json_schema": {
            "schema": pyobject_to_json(&ensure_strict_json_schema(schema)?)?,
            "name": name,
            "strict": true,
        },
    }))
}
