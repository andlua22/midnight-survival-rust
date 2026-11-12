//! Compact inspection of the existing Rust TypeTree inspector's player reports.

use serde_json::{Value, json};

fn shape(value: &Value, depth: usize) -> Value {
    match value {
        Value::Object(fields) if depth > 0 => Value::Object(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), shape(value, depth - 1)))
                .collect(),
        ),
        Value::Object(fields) => json!({"keys": fields.keys().collect::<Vec<_>>()}),
        Value::Array(values) => {
            json!({"count": values.len(), "first": values.first().map(|value| shape(value, depth.saturating_sub(1)))})
        }
        _ => value.clone(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("expected JSON inspection report")?;
    let selector = args.next();
    let report: Value = serde_json::from_reader(std::fs::File::open(path)?)?;
    for asset in report["assets"].as_array().ok_or("missing assets")? {
        println!(
            "{}",
            json!({"cab": asset["cab"], "class_counts": asset["class_counts"], "externals": asset["externals"]})
        );
        let objects = asset["objects"].as_array().ok_or("missing objects")?;
        if let Some(selector) = &selector {
            for object in objects.iter().filter(|object| {
                object["path_id"] == selector.as_str()
                    || object["properties"]["m_Name"] == selector.as_str()
            }) {
                println!("{}", object);
            }
        } else {
            println!(
                "{}",
                json!({"objects": objects.iter().map(|object| json!({"path_id": object["path_id"], "class_id":object["class_id"], "name":object["properties"]["m_Name"]})).collect::<Vec<_>>()})
            );
            let mut classes = std::collections::BTreeSet::new();
            for object in objects {
                if classes.insert(object["class_id"].as_i64()) {
                    println!(
                        "{}",
                        json!({"class_id":object["class_id"], "path_id":object["path_id"], "properties": shape(&object["properties"], 5)})
                    );
                }
            }
        }
    }
    Ok(())
}
