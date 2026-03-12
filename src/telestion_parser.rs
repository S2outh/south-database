
use ciborium::Value;
use ciborium_io::Read;

use questdb::ingress::{
    Buffer, TimestampMicros
};

fn add_value(buffer: &mut Buffer, name: &str, value: &Value) -> Result<(), ParsingError> {
    match value {
        Value::Bool(b) => {
            buffer.column_bool(name, b.to_owned())
                .map_err(ParsingError::QuestDBErr)?;
        },
        Value::Integer(n) => {
            buffer.column_i64(name, (*n).try_into()
                .map_err(|_| ParsingError::UnsupportedType)?)
                .map_err(ParsingError::QuestDBErr)?;
        },
        Value::Float(n) => {
            buffer.column_f64(name, *n)
                .map_err(ParsingError::QuestDBErr)?;
        },
        Value::Text(s) => {
            buffer.column_str(name, s)
                .map_err(ParsingError::QuestDBErr)?;
        },
        Value::Bytes(b) => {
            for (i, entry) in b.iter().enumerate() {
                buffer.column_i64(&format!("{}_{}", name, i) as &str, (*entry).into())
                    .map_err(ParsingError::QuestDBErr)?;
            }
        },
        Value::Array(a) => {
            for (i, entry) in a.iter().enumerate() {
                add_value(buffer, &format!("{}_{}", name, i), entry)?;
            }
        },
        Value::Map(o) => {
            for entry in o.iter() {
                let Value::Text(key) = &entry.0 else {
                    return Err(ParsingError::UnsupportedFormat)
                };
                add_value(buffer, &format!("{}_{}", name, key), &entry.1)?;
            }
        },
        _ => return Err(ParsingError::UnsupportedType),
    }
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParsingError {
    UnsupportedFormat,
    UnsupportedType,
    QuestDBErr(questdb::Error),
    SerdeErr(ciborium::de::Error<<&'static [u8] as Read>::Error>),
}

#[derive(serde::Deserialize)]
struct TelestionMsg {
    timestamp: i64,
    value: Value,
}

pub fn to_buffer(topic: &str, content: &[u8]) -> Result<Buffer, ParsingError> {
    
    let msg: TelestionMsg = ciborium::from_reader(content).map_err(ParsingError::SerdeErr)?;
    let topic_tail = topic.split(".").last().unwrap();

    let mut buffer = Buffer::new();

    buffer.table(topic).map_err(ParsingError::QuestDBErr)?;

    add_value(&mut buffer, &topic_tail, &msg.value)?;

    buffer.at(TimestampMicros::new(msg.timestamp))
        .map_err(ParsingError::QuestDBErr)?;

    println!("[INFO] parsed topic: {}", topic);

    Ok(buffer)
}

