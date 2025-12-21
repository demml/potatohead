use potato_type::prompt::Message;
use serde_json::Value;
use std::collections::HashMap;
pub type Context = (HashMap<String, Vec<Message>>, Value, Option<Value>);
