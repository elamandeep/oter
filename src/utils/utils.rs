use dbase::FieldValue;
use serde_json::{json, Value};
use std::{fs, io::Write};
use uuid::Uuid;

pub fn parse_dbase_value(field_value: FieldValue) -> Value {
    match field_value {
        FieldValue::Character(c) => {
            json!(Some(c))
        }
        FieldValue::Numeric(n) => json!(Some(n)),
        FieldValue::Logical(l) => json!(Some(l)),
        FieldValue::Date(d) => {
            if let Some(d) = d {
                json!(d.to_unix_days())
            } else {
                json!("")
            }
        }
        FieldValue::Float(f) => json!(Some(f)),
        FieldValue::Integer(i) => json!(Some(i)),
        FieldValue::Currency(c) => json!(c),
        FieldValue::DateTime(d) => {
            json!(d.to_unix_timestamp())
        }
        FieldValue::Double(d) => json!(d),
        FieldValue::Memo(m) => json!(m),
    }
}

pub fn save(extension: &str, content: String) {
    let id = Uuid::new_v4();
    let filename = id.as_simple().to_string();
    let file_name = format!("{}.{}", &filename[0..12], extension);
    let mut file = fs::File::create(file_name).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
