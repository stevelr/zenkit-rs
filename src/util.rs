use crate::{Error, Result};
use serde::de::{Deserializer, Visitor};
use std::{fmt, io::Write};

const TRACE_OUTPUT: &str = "/tmp";

/*
/// Logging level for debug output
#[derive(PartialEq, Debug)]
pub enum LogLevel {
    /// Info level, the default level
    Info,
    /// Debug level - additional verbosity
    Debug,
    /// Trace level - logs all messages
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl FromStr for LogLevel {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "info" | "Info" | "INFO" => Ok(LogLevel::Info),
            "debug" | "Debug" | "DEBUG" => Ok(LogLevel::Debug),
            "trace" | "Trace" | "TRACE" => Ok(LogLevel::Trace),
            _ => Err(Error::ParseError(format!("Invalid loglevel: {}", s))),
        }
    }
}
*/

/// write json object to log file (in trace mode)
pub fn log_object(tstamp: &str, name: &str, data: &[u8]) -> Result<String, Error> {
    let fname = format!("{}/log_{}_{}.json", TRACE_OUTPUT, tstamp, name);
    let mut file = std::fs::File::create(&fname)?;
    file.write_all(data)?;
    Ok(fname)
}

/// Deserializer for value that can be an int, float, or string
// I've seen sortOrder in Entry objects returned
//      as int (small positive: 1..), float (1.399964), and string (quoted negative num "-99")
// So I needed a custom deserializer that first parses to serde_json Value,
// then converted to an f32. I'm assuming precision of f64 is not needed.
pub(crate) fn f32_or_str<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(AnyF32Visitor)
}

struct AnyF32Visitor;

impl<'de> Visitor<'de> for AnyF32Visitor {
    type Value = f32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an i64 integer or string representation of i64")
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(v as f32)
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(v as f32)
    }

    fn visit_i32<E: serde::de::Error>(self, v: i32) -> Result<Self::Value, E> {
        Ok(v as f32)
    }

    fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Self::Value, E> {
        Ok(v as f32)
    }

    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Ok(v as f32)
    }

    fn visit_f32<E: serde::de::Error>(self, v: f32) -> Result<Self::Value, E> {
        Ok(v as f32)
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        match v.parse::<f32>() {
            Ok(num) => Ok(num),
            Err(_) => Err(E::custom(format!("not a valid float value: {}", v))),
        }
    }
}

pub(crate) fn join<T: AsRef<str>>(sep: &str, arr: &[T]) -> String {
    if arr.is_empty() {
        return String::from("");
    }
    let mut result = arr.get(0).unwrap().as_ref().to_string();
    for s in &arr[1..] {
        result.push_str(sep);
        result.push_str(s.as_ref());
    }
    result
}

/// quick test to determine if string is valid uuid
// avoids dependency on regex lib
pub(crate) fn is_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    for buf in s.split('-') {
        match buf.len() {
            8 => {
                if u64::from_str_radix(&buf, 16).is_err() {
                    return false;
                }
            }
            4 => {
                if u32::from_str_radix(&buf, 16).is_err() {
                    return false;
                }
            }
            12 => {
                if u64::from_str_radix(&buf[..6], 16).is_err()
                    || u64::from_str_radix(&buf[6..], 16).is_err()
                {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::{is_uuid, join};

    #[test]
    fn test_join() -> Result<(), ()> {
        assert_eq!(&join(",", &Vec::<String>::new()), "", "empty array");
        assert_eq!(&join(",", &vec!["ab"]), "ab", "1-list");
        assert_eq!(&join(",", &vec!["ab", "cd"]), "ab,cd", "2-list");
        assert_eq!(&join(",", &vec!["ab", "cd", "ef"]), "ab,cd,ef", "3-list");
        assert_eq!(&join("", &vec!["ab", "cd"]), "abcd", "empty sep");
        assert_eq!(&join("/", &vec!["", "home", "user"]), "/home/user", "mixed");
        Ok(())
    }

    #[test]
    fn test_uuid() {
        for s in vec![
            "00000000-0000-0000-0000-000000000000",
            "abcdef00-1234-5678-9999-abcdefabcdef",
        ]
        .iter()
        {
            assert_eq!(is_uuid(s), true, "expect good: {}", s);
        }

        for s in vec![
            "",
            "00000000",
            "zzzzzzzz-0000-0000-0000-000000000000",
            "000000000000000000000000000000000000",
        ]
        .iter()
        {
            assert_ne!(is_uuid(s), true, "expect bad: {}", s);
        }
    }
}
