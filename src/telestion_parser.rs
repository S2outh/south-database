
use serde_json::Value;

use questdb::ingress::{
    Buffer, TimestampMicros
};

const MILLIS: i64 = 1000;

fn add_value(buffer: &mut Buffer, name: &str, value: &Value, tp: &Value) -> Result<(), ParsingError> {
    match value {
        Value::Null => return Err(ParsingError::UnsupportedFormat),
        Value::Bool(b) => {
            buffer.column_bool(name, b.to_owned())
                .map_err(|e| ParsingError::QuestDBErr(e))?;
        },
        Value::Number(n) => {
            match tp.as_str().ok_or(ParsingError::UnsupportedType)? {
                "int8" | "int16" | "int32" | "int64" =>
                    buffer.column_i64(name, n.as_i64()
                        .ok_or(ParsingError::UnsupportedFormat)?)
                        .map_err(|e| ParsingError::QuestDBErr(e))?,

                "uint8" | "uint16" | "uint32" | "uint64" =>
                    buffer.column_i64(name, n.as_u64()
                        .ok_or(ParsingError::UnsupportedFormat)? as i64)
                        .map_err(|e| ParsingError::QuestDBErr(e))?,


                "float32" | "float64" =>
                    buffer.column_f64(name, n.as_f64()
                        .ok_or(ParsingError::UnsupportedFormat)?)
                        .map_err(|e| ParsingError::QuestDBErr(e))?,

                _ => return Err(ParsingError::UnsupportedType),
            };
        }
        Value::String(s) => {
            buffer.column_str(name, s).map_err(|e| ParsingError::QuestDBErr(e))?;
        },
        Value::Array(a) => {
            for (i, entry) in a.iter().enumerate() {
                add_value(buffer, &format!("{}_{}", name, i), entry, tp)?;
            }
        },
        Value::Object(o) => {
            for entry in o.iter() {
                let tpv = if let Value::Object(tpo) = tp { &tpo[entry.0] } else { tp };
                add_value(buffer, &format!("{}_{}", name, entry.0), entry.1, tpv)?;
            }
        },
    }
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParsingError {
    UnsupportedFormat,
    UnsupportedType,
    QuestDBErr(questdb::Error),
    SerdeErr(serde_json::Error),
}

#[derive(serde::Deserialize)]
struct TelestionMsg {
    timestamp: i64,
    value: Value,
    #[serde(rename = "type")]
    tp: Value,
}

pub fn to_buffer(topic: &str, content: &[u8]) -> Result<Buffer, ParsingError> {
    
    let msg = serde_json::from_slice::<TelestionMsg>(content).map_err(|e| ParsingError::SerdeErr(e))?;
    let topic_tail = topic.split(".").last().unwrap();

    let mut buffer = Buffer::new();

    buffer.table(topic).map_err(|e| ParsingError::QuestDBErr(e))?;

    add_value(&mut buffer, &topic_tail, &msg.value, &msg.tp)?;

    buffer.at(TimestampMicros::new(msg.timestamp * MILLIS))
        .map_err(|e| ParsingError::QuestDBErr(e))?;

    Ok(buffer)
}

