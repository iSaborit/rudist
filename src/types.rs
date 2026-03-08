#[derive(Debug, Clone, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<RespValue>),
    Null,
}

impl RespValue {
    pub fn as_command(&self) -> Option<String> {
        if let Self::BulkString(bytes) = self {
            return Some(String::from_utf8_lossy(bytes).to_uppercase());
        } else {
            return None;
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(value) => format!("+{}\r\n", value).into_bytes(),
            RespValue::Error(e) => format!("-{}\r\n", e).into_bytes(),
            RespValue::Integer(i) => format!(":{}\r\n", i).into_bytes(),
            RespValue::BulkString(b) => {
                let mut res = format!("${}\r\n", b.len()).into_bytes();
                res.extend_from_slice(b);
                res.extend_from_slice(b"\r\n");

                res
            }
            RespValue::Null => format!("-1\r\n").into_bytes(),
            RespValue::Array(array) => {
                let mut res = format!("*{}\r\n",array.len()).into_bytes();
                for item in array {
                    res.extend_from_slice(&item.serialize());
                }

                res
            }
            _ => todo!(),
        }
    }
}
