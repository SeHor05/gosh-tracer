use std::collections::HashMap;

use serde_json::{json, Value};
use tera::Error;

pub fn shorten_string(value: &Value, kwargs: &HashMap<String, Value>) -> Result<Value, Error> {
    let start = kwargs.get("start").unwrap().as_u64().unwrap_or_default();
    let ustart = usize::try_from(start).expect("Error in start index");

    let end = kwargs.get("end").unwrap().as_u64().unwrap_or_default();
    let uend = usize::try_from(end).expect("Error in end index");

    let default_delimiter = json!("...");
    let delimiter = kwargs
        .get("delimiter")
        .unwrap_or(&default_delimiter)
        .as_str()
        .unwrap_or("...");

    let val = value.as_str().unwrap();
    if val.len() < ustart + uend {
        return Ok(json!(val));
    }

    let left = &val[0..ustart];
    let right = &val[val.len() - uend..];
    Ok(json!(format!("{}{}{}", left, delimiter, right)))
}
