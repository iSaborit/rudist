use std::io::{BufRead, BufReader, Read};

use crate::types::RespValue;

pub struct RespParser<R: Read> {
    reader: BufReader<R>,
    line_buffer: String,
}

impl<R: Read> RespParser<R> {
    pub fn new(inner: R) -> Self {
        Self {
            reader: BufReader::new(inner),
            line_buffer: String::with_capacity(64),
        }
    }

    pub fn parse(&mut self) -> Result<RespValue, String> {
        let mut prefix = [0u8; 1];
        self.reader
            .read_exact(&mut prefix)
            .map_err(|e| e.to_string())?;

        match prefix[0] {
            b'+' => self.parse_simple_string(),
            b'$' => self.parse_bulk_string(),
            b'*' => self.parse_array(),
            _ => Err(format!("Tipo de dato no soportado: {}", prefix[0])),
        }
    }

    pub fn parse_simple_string(&mut self) -> Result<RespValue, String> {
        self.line_buffer.clear();
        self.reader
            .read_line(&mut self.line_buffer)
            .map_err(|e| e.to_string())?;

        Ok(RespValue::SimpleString(self.line_buffer.trim().to_string()))
    }

    pub fn parse_bulk_string(&mut self) -> Result<RespValue, String> {
        self.line_buffer.clear();
        self.reader
            .read_line(&mut self.line_buffer)
            .map_err(|e| e.to_string())?;

        // Asumiendo que la línea incluye e
        let index = self
            .line_buffer
            .find("\r\n")
            .ok_or("Invalid request, \\r\\n missing")?;
        let length_str = &self.line_buffer[0..index];
        let length = length_str.parse::<usize>().map_err(|e| e.to_string())?;

        let mut value = vec![0; length];

        self.reader
            .read_exact(&mut value)
            .map_err(|e| e.to_string())?;

        let mut trailing_control = [0u8; 2];
        self.reader
            .read_exact(&mut trailing_control)
            .map_err(|e| e.to_string())?;

        if &trailing_control != b"\r\n" {
            return Err(format!("Invalid format, \\r\\n missing."));
        }

        Ok(RespValue::BulkString(value))
    }

    fn parse_array(&mut self) -> Result<RespValue, String> {
        self.line_buffer.clear();
        self.reader
            .read_line(&mut self.line_buffer)
            .map_err(|e| e.to_string())?;

        let index = self
            .line_buffer
            .find("\r\n")
            .ok_or("Invalid request, \\r\\n missing")?;

        if self.line_buffer.starts_with("-1") {
            return Ok(RespValue::Null);
        }

        let count_str = &self.line_buffer[..index];

        // 3. Manejo de Null Array (*-1\r\n)
        if count_str == "-1" {
            return Ok(RespValue::Null);
        }

        let mut value: Vec<RespValue> = Vec::new();

        let length = count_str.parse::<i64>().map_err(|e| e.to_string())?;

        for _ in 0..length {
            let item = self.parse()?;
            value.push(item);
        }

        Ok(RespValue::Array(value))
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
