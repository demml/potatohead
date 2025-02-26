use potato_error::PotatoHeadError;
use potato_tools::pydict_to_json;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict};
use serde_json::{json, Value};

fn ensure_strict_json_schema<'py>(schema: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    if !schema.is_instance_of::<PyDict>() {
        return Err(PotatoHeadError::new_err("Schema is not a dictionary"));
    }

    let schema = schema.downcast::<PyDict>()?;

    let schema_type = schema.get_item("type")?.unwrap().extract::<String>()?;

    let additional_props = schema.get_item("additionalProperties")?;

    if schema_type == "object" && additional_props.is_none() {
        schema.set_item("additionalProperties", false)?;
    }

    Ok(schema.clone())
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

    let strict_schema = ensure_strict_json_schema(schema)
        .map_err(|e| PotatoHeadError::new_err(format!("Failed to ensure strict schema: {}", e)))?;

    let converted_schema = pydict_to_json(&strict_schema).map_err(|e| {
        PotatoHeadError::new_err(format!("Failed to convert schema to JSON: {}", e))
    })?;

    // Create JSON response
    Ok(json!({
        "type": "json_schema",
        "json_schema": {
            "schema": converted_schema,
            "name": name,
            "strict": true,
        },
    }))
}
