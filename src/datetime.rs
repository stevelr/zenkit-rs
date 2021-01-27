//! implementation of custom datetime type so we can use deserialization
//! when the date has date only and no time. In that case, we coerce it to a time with 00:00:00,
//! and force its timezone to Utc for consitency. The application can easily
//! change timezeone by calling with_timezone(tz).

// use and re-export the Utc timezone
pub use chrono::Utc;
use serde::{de, Serialize};
use std::{fmt, ops::Deref, str::FromStr};

/// Variation on [`chrono::DateTime`](chrono::DateTime) that can parse and deserialize
/// a Zenkit date with or without time.
/// The value is always converted to Utc during parse/deserialization, for consistency,
/// but the app can change timezone by calling with_timezone(tz).
/// The type is declared generic over Timezone Tz, and it should work with different timezones for
/// most operations; but parsing from string and deserializing will always create a <Utc> variant.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DateTime<Tz: chrono::offset::TimeZone>(chrono::DateTime<Tz>);

/// Deref implementation so chrono::DateTime methods should be nearly seamless
impl<Tz: chrono::offset::TimeZone> Deref for DateTime<Tz> {
    type Target = chrono::DateTime<Tz>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Implement Display
impl<Tz: chrono::offset::TimeZone> fmt::Display for DateTime<Tz>
where
    Tz::Offset: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Implement Debug
impl<Tz: chrono::offset::TimeZone> fmt::Debug for DateTime<Tz>
where
    Tz::Offset: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct DateTimeVisitor;

impl<'de> de::Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a formatted date or date and time string")
    }

    fn visit_str<E>(self, value: &str) -> Result<DateTime<Utc>, E>
    where
        E: de::Error,
    {
        parse_date(value).map_err(|err| E::custom(format!("{}", err)))
    }
}

impl<'de> de::Deserialize<'de> for DateTime<Utc> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeVisitor)
    }
}

impl Serialize for DateTime<Utc> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Parse date into DateTime, using either a full datetime format. or just the date (YYYY-MM-DD)
fn parse_date(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    let dt: chrono::DateTime<Utc> = if s.len() == 10 {
        let nd = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
        chrono::DateTime::from_utc(nd.and_hms(0, 0, 0), Utc)
    } else {
        s.parse::<chrono::DateTime<chrono::FixedOffset>>()
            .map(|dt| dt.with_timezone(&Utc))?
    };
    Ok(DateTime(dt))
}

impl FromStr for DateTime<Utc> {
    type Err = chrono::ParseError;

    /// Parse date into DateTime, using either a full datetime format. or just the date (YYYY-MM-DD).
    /// If the date has a timezone, it will be converted to equivalent time in Utc.
    fn from_str(s: &str) -> chrono::ParseResult<DateTime<Utc>> {
        parse_date(s)
    }
}

impl From<chrono::DateTime<Utc>> for DateTime<Utc> {
    fn from(d: chrono::DateTime<Utc>) -> Self {
        DateTime(d)
    }
}

/// test serialize and deserialize implementation
#[test]
fn test_datetime_ser_deser() {
    #[derive(Debug, Serialize, serde::Deserialize)]
    struct DateStruct {
        date: DateTime<Utc>,
    }

    let val: DateStruct = serde_json::from_str(r#"{ "date": "2020-01-01" }"#).unwrap();
    assert_eq!(
        format!("date: {:?}", val.date),
        "date: 2020-01-01 00:00:00 UTC"
    );

    let json = serde_json::to_string(&val).unwrap();
    assert_eq!(json, r#"{"date":"2020-01-01T00:00:00Z"}"#);
}

/// test impl Debug
#[test]
fn test_datetime_debug() {
    let dt = "2020-01-01T01:02:03Z".parse::<DateTime<Utc>>().unwrap();
    assert_eq!(format!("{:?}", dt), "2020-01-01 01:02:03 UTC");
}

/// test impl Display
#[test]
fn test_datetime_display() {
    let dt = "2020-01-01T01:02:03Z".parse::<DateTime<Utc>>().unwrap();
    assert_eq!(format!("{}", dt), "2020-01-01 01:02:03 UTC");
}

/// test parse iso8601 format
#[test]
fn test_datetime_parse_iso8601() {
    let dt = "2020-01-01T01:02:03Z".parse::<DateTime<Utc>>().unwrap();
    assert_eq!(format!("{}", dt), "2020-01-01 01:02:03 UTC");
    assert_eq!(format!("{}", dt.0), "2020-01-01 01:02:03 UTC");
}

/// test parse short date "YYYY-MM-DD"
#[test]
fn test_datetime_parse_short() {
    let dt = "2020-01-01".parse::<DateTime<Utc>>().unwrap();
    assert_eq!(format!("{}", dt), "2020-01-01 00:00:00 UTC");
    assert_eq!(format!("{}", dt.0), "2020-01-01 00:00:00 UTC");
}
