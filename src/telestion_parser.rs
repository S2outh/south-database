
use serde_cbor::Value;

use questdb::ingress::{
    Buffer, TimestampMicros
};

fn add_value(buffer: &mut Buffer, name: &str, value: &Value) -> Result<(), ParsingError> {
    match value {
        Value::Bool(b) => {
            buffer.column_bool(name, b.to_owned())
                .map_err(|e| ParsingError::QuestDBErr(e))?;
        },
        Value::Integer(n) => {
            buffer.column_i64(name, *n as i64)
                .map_err(|e| ParsingError::QuestDBErr(e))?;
        },
        Value::Float(n) => {
            buffer.column_f64(name, *n)
                .map_err(|e| ParsingError::QuestDBErr(e))?;
        },
        Value::Text(s) => {
            buffer.column_str(name, s)
                .map_err(|e| ParsingError::QuestDBErr(e))?;
        },
        Value::Array(a) => {
            for (i, entry) in a.iter().enumerate() {
                add_value(buffer, &format!("{}_{}", name, i), entry)?;
            }
        },
        Value::Map(o) => {
            for entry in o.iter() {
                let Value::Text(key) = entry.0 else {
                    return Err(ParsingError::UnsupportedFormat)
                };
                add_value(buffer, &format!("{}_{}", name, key), entry.1)?;
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
    SerdeErr(serde_cbor::Error),
}

#[derive(serde::Deserialize)]
struct TelestionMsg {
    timestamp: i64,
    value: Value,
}

pub fn to_buffer(topic: &str, content: &[u8]) -> Result<Buffer, ParsingError> {
    
    let msg = serde_cbor::from_slice::<TelestionMsg>(content).map_err(|e| ParsingError::SerdeErr(e))?;
    let topic_tail = topic.split(".").last().unwrap();

    let mut buffer = Buffer::new();

    buffer.table(topic).map_err(|e| ParsingError::QuestDBErr(e))?;

    add_value(&mut buffer, &topic_tail, &msg.value)?;

    const MILLIS: i64 = 1000;
    buffer.at(TimestampMicros::new(msg.timestamp * MILLIS))
        .map_err(|e| ParsingError::QuestDBErr(e))?;

    Ok(buffer)
}

