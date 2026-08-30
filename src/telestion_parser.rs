use ciborium::Value;
use ciborium_io::Read;

use std::{fmt, error};

use questdb::ingress::{
    Buffer, TimestampMicros
};

fn add_value(buffer: &mut Buffer, name: &str, value: &Value) -> Result<()> {
    match value {
        Value::Bool(b) => {
            buffer.column_bool(name, b.to_owned())?;
        },
        Value::Integer(n) => {
            buffer.column_i64(name, (*n).try_into()
                .map_err(|_| ParsingError::UnsupportedType)?)?;
        },
        Value::Float(n) => {
            buffer.column_f64(name, *n)?;
        },
        Value::Text(s) => {
            buffer.column_str(name, s)?;
        },
        Value::Bytes(b) => {
            for (i, entry) in b.iter().enumerate() {
                buffer.column_i64(&format!("{}_{}", name, i) as &str, (*entry).into())?;
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

type CiboriumErr = ciborium::de::Error<<&'static [u8] as Read>::Error>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParsingError {
    UnsupportedFormat,
    UnsupportedType,
    QuestDBErr(questdb::Error),
    SerdeErr(CiboriumErr),
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat => write!(f, "Unsupported format"),
            Self::UnsupportedType => write!(f, "Unsupported format"),
            Self::QuestDBErr(e) => write!(f, "DB error: {}", e),
            Self::SerdeErr(e) => write!(f, "Deserialization error: {}", e),
        }
    }
}

from_inner_err!(ParsingError,
    QuestDBErr, questdb::Error,
    SerdeErr, CiboriumErr,
);

impl error::Error for ParsingError {}

type Result<T> = std::result::Result<T, ParsingError>;

#[derive(serde::Deserialize)]
struct TelestionMsg {
    timestamp: i64,
    value: Value,
}

pub fn to_buffer(topic: &str, content: &[u8]) -> Result<Buffer> {
    
    let msg: TelestionMsg = ciborium::from_reader(content)?;
    let topic_tail = topic.split(".").last().unwrap();

    let mut buffer = Buffer::new();

    buffer.table(topic)?;

    add_value(&mut buffer, &topic_tail, &msg.value)?;

    buffer.at(TimestampMicros::new(msg.timestamp))?;

    println!("[INFO] parsed topic: {}", topic);

    Ok(buffer)
}

