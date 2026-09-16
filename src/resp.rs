use std::io::{BufRead, BufReader, ErrorKind::UnexpectedEof, Read};

use crate::types::RespValue;

pub struct RespParser<R: Read> {
    reader: BufReader<R>,
    line_buffer: String,
}

#[derive(Debug)]
pub enum RespError {
    Protocol(String),
    Io(std::io::Error),
    Eof,
}

impl std::error::Error for RespError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RespError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespError::Protocol(e) => write!(f, "protocol: {e}"),
            RespError::Io(e) => write!(f, "io: {e}"),
            RespError::Eof => write!(f, "eof"),
        }
    }
}

impl From<std::io::Error> for RespError {
    fn from(value: std::io::Error) -> Self {
        RespError::Io(value)
    }
}

impl<R: Read> RespParser<R> {
    pub fn new(inner: R) -> Self {
        Self {
            reader: BufReader::new(inner),
            line_buffer: String::with_capacity(64),
        }
    }

    pub fn get_command(&self) -> String {
        self.line_buffer.clone()
    }

    pub fn parse(&mut self) -> Result<RespValue, RespError> {
        let mut prefix = [0u8; 1];
        self.reader.read_exact(&mut prefix).map_err(|e| {
            if e.kind() == UnexpectedEof {
                RespError::Eof
            } else {
                RespError::Io(e)
            }
        })?;

        match prefix[0] {
            b'+' => self.parse_simple_string(),
            b'$' => self.parse_bulk_string(),
            b'*' => self.parse_array(),
            b':' => self.parse_number(),
            b'-' => self.parse_error(),

            _ => Err(RespError::Protocol(format!(
                "Tipo de dato no soportado: {}",
                prefix[0]
            ))),
        }
    }

    pub fn parse_simple_string(&mut self) -> Result<RespValue, RespError> {
        self.line_buffer.clear();
        self.reader.read_line(&mut self.line_buffer)?;

        Ok(RespValue::SimpleString(self.line_buffer.trim().to_string()))
    }

    pub fn parse_bulk_string(&mut self) -> Result<RespValue, RespError> {
        self.line_buffer.clear();
        self.reader.read_line(&mut self.line_buffer)?;

        // Asumiendo que la línea incluye e
        let index = self.line_buffer.find("\r\n").ok_or(RespError::Protocol(
            "Invalid request, \\r\\n missing".to_string(),
        ))?;

        let length_str = &self.line_buffer[0..index];
        let length_t = length_str
            .parse::<i64>()
            .map_err(|e| RespError::Protocol(e.to_string()))?;

        if length_t == -1 {
            return Ok(RespValue::Null);
        } else if length_t < -1 {
            return Err(RespError::Protocol("invalid value".to_string()));
        }

        let length = length_t as usize;

        let mut value = vec![0; length];

        self.reader.read_exact(&mut value)?;

        let mut trailing_control = [0u8; 2];
        self.reader.read_exact(&mut trailing_control)?;

        if &trailing_control != b"\r\n" {
            return Err(RespError::Protocol(
                "Invalid format, \\r\\n missing.".to_string(),
            ));
        }

        Ok(RespValue::BulkString(value))
    }

    fn parse_array(&mut self) -> Result<RespValue, RespError> {
        self.line_buffer.clear();
        self.reader.read_line(&mut self.line_buffer)?;

        let index = self.line_buffer.find("\r\n").ok_or(RespError::Protocol(
            "Invalid request, \\r\\n missing".to_string(),
        ))?;

        let count_str = &self.line_buffer[..index];

        let mut value: Vec<RespValue> = Vec::new();

        let length = count_str
            .parse::<i64>()
            .map_err(|e| RespError::Protocol(e.to_string()))?;

        if length == -1 {
            return Ok(RespValue::Null);
        } else if length < -1 {
            return Err(RespError::Protocol("invalid array length".to_string()));
        }

        for _ in 0..length {
            let item = self.parse()?;
            value.push(item);
        }

        Ok(RespValue::Array(value))
    }

    fn parse_error(&mut self) -> Result<RespValue, RespError> {
        self.line_buffer.clear();
        self.reader.read_line(&mut self.line_buffer)?;

        Ok(RespValue::Error(self.line_buffer.trim().to_string()))
    }

    fn parse_number(&mut self) -> Result<RespValue, RespError> {
        self.line_buffer.clear();
        self.reader.read_line(&mut self.line_buffer)?;

        Ok(RespValue::Integer(
            self.line_buffer
                .trim()
                .parse::<i64>()
                .map_err(|e| RespError::Protocol(e.to_string()))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple_string() {
        let data = "+OK\r\n";
        let mut parser = RespParser::new(Cursor::new(data));

        let result = parser.parse().unwrap();

        match result {
            RespValue::SimpleString(s) => assert_eq!(s, "OK"),
            _ => panic!("Se esperaba un SimpleString"),
        }
    }

    #[test]
    fn test_parse_integer() {
        let data = ":123\r\n";
        let mut parser = RespParser::new(Cursor::new(data));
        let result = parser.parse().unwrap();

        assert_eq!(result, RespValue::Integer(123));
    }

    #[test]
    fn test_parse_integer_on_array() {
        let data = "*2\r\n:-123\r\n$4\r\nPING\r\n";
        let mut parser = RespParser::new(Cursor::new(data));
        let result = parser.parse().unwrap();

        let expected = vec![
            RespValue::Integer(-123),
            RespValue::BulkString(b"PING".to_vec()),
        ];

        assert_eq!(result, RespValue::Array(expected));
    }

    #[test]
    fn test_parse_simple_void_string() {
        let data = "+\r\n";
        let mut parser = RespParser::new(Cursor::new(data));

        let result = parser.parse().unwrap();

        match result {
            RespValue::SimpleString(s) => assert_eq!(s, ""),
            _ => panic!("Se esperaba un SimpleString"),
        }
    }

    #[test]
    fn test_parse_bulk_string() {
        let data = "$5\r\nhello\r\n";
        let mut parser = RespParser::new(Cursor::new(data));

        let result = parser.parse().unwrap();

        match result {
            RespValue::BulkString(s) => assert_eq!(s, b"hello".to_vec()),
            _ => panic!("Se esperaba bulk string!"),
        }

        let data = "$5\r\nhe\rlo\r\n";
        let mut parser = RespParser::new(Cursor::new(data));

        let result = parser.parse().unwrap();

        match result {
            RespValue::BulkString(s) => assert_eq!(s, b"he\rlo".to_vec()),
            _ => panic!("Se esperaba bulk string!"),
        }
    }

    #[test]
    fn test_parse_bulk_string_fail() {
        let data = "$6\r\nhello\r\n";
        let mut parser = RespParser::new(Cursor::new(data));

        let result = parser.parse();

        assert!(result.is_err());

        let data = "$4\r\nhello\r\n";
        let mut parser = RespParser::new(Cursor::new(data));

        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array() {
        let data = "*2\r\n$4\r\nPING\r\n$2\r\nhi\r\n";
        let mut parser = RespParser::new(Cursor::new(data));

        let result = parser.parse().unwrap();

        let expected = vec![
            RespValue::BulkString(b"PING".to_vec()),
            RespValue::BulkString(b"hi".to_vec()),
        ];

        match result {
            RespValue::Array(s) => assert_eq!(s, expected),
            _ => panic!("Se esperaba bulk string!"),
        }
    }
}
