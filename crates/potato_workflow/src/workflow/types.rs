use potato_prompt::Message;
use serde_json::Value;
use std::collections::HashMap;
pub type Context = (HashMap<String, Vec<Message>>, Value, Option<Value>);
