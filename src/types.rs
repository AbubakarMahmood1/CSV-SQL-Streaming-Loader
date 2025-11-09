//! SQL type system for schema inference

use chrono::NaiveDateTime;
use std::fmt;

/// Represents PostgreSQL data types we can infer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SqlType {
    Null,
    Boolean,
    SmallInt,
    Integer,
    BigInt,
    Real,
    DoublePrecision,
    Timestamp,
    Date,
    Text,
}

impl SqlType {
    /// Get the PostgreSQL type name
    pub fn to_sql(&self) -> &str {
        match self {
            SqlType::Null => "TEXT", // Default to TEXT for NULL columns
            SqlType::Boolean => "BOOLEAN",
            SqlType::SmallInt => "SMALLINT",
            SqlType::Integer => "INTEGER",
            SqlType::BigInt => "BIGINT",
            SqlType::Real => "REAL",
            SqlType::DoublePrecision => "DOUBLE PRECISION",
            SqlType::Timestamp => "TIMESTAMP",
            SqlType::Date => "DATE",
            SqlType::Text => "TEXT",
        }
    }

    /// Infer type from a string value
    pub fn infer_from_str(value: &str) -> Self {
        // Empty or null-like values
        if value.is_empty() || value.eq_ignore_ascii_case("null") || value.eq_ignore_ascii_case("\\N") {
            return SqlType::Null;
        }

        // Boolean
        if let Ok(_) = value.parse::<bool>() {
            return SqlType::Boolean;
        }

        // Try integers (from smallest to largest)
        if let Ok(_val) = value.parse::<i16>() {
            return SqlType::SmallInt;
        }
        if let Ok(_val) = value.parse::<i32>() {
            return SqlType::Integer;
        }
        if let Ok(_val) = value.parse::<i64>() {
            return SqlType::BigInt;
        }

        // Try floats
        if let Ok(val) = value.parse::<f32>() {
            if !val.is_infinite() && !val.is_nan() {
                return SqlType::Real;
            }
        }
        if let Ok(val) = value.parse::<f64>() {
            if !val.is_infinite() && !val.is_nan() {
                return SqlType::DoublePrecision;
            }
        }

        // Try timestamp formats
        if Self::is_timestamp(value) {
            return SqlType::Timestamp;
        }

        // Try date formats
        if Self::is_date(value) {
            return SqlType::Date;
        }

        // Default to text
        SqlType::Text
    }

    /// Check if value looks like a timestamp
    fn is_timestamp(value: &str) -> bool {
        // Common timestamp formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y/%m/%d %H:%M:%S",
            "%d-%m-%Y %H:%M:%S",
            "%m/%d/%Y %H:%M:%S",
        ];

        formats.iter().any(|fmt| {
            NaiveDateTime::parse_from_str(value, fmt).is_ok()
        })
    }

    /// Check if value looks like a date
    fn is_date(value: &str) -> bool {
        // Common date formats
        let formats = [
            "%Y-%m-%d",
            "%Y/%m/%d",
            "%d-%m-%Y",
            "%m/%d/%Y",
            "%d/%m/%Y",
        ];

        formats.iter().any(|fmt| {
            chrono::NaiveDate::parse_from_str(value, fmt).is_ok()
        })
    }

    /// Merge two types to find the most general type
    pub fn merge(&self, other: &SqlType) -> SqlType {
        use SqlType::*;

        // Ordering: Null < Boolean < SmallInt < Integer < BigInt < Real < DoublePrecision < Timestamp < Date < Text
        // If types differ, promote to the more general type

        match (self, other) {
            // If either is Text, result is Text
            (Text, _) | (_, Text) => Text,

            // If either is Null, use the other
            (Null, x) | (x, Null) => x.clone(),

            // Same types
            (a, b) if a == b => a.clone(),

            // Numeric promotions
            (SmallInt, Integer) | (Integer, SmallInt) => Integer,
            (SmallInt, BigInt) | (BigInt, SmallInt) => BigInt,
            (Integer, BigInt) | (BigInt, Integer) => BigInt,

            // Any int with any float -> float
            (SmallInt | Integer | BigInt, Real | DoublePrecision) |
            (Real | DoublePrecision, SmallInt | Integer | BigInt) => DoublePrecision,

            // Float promotions
            (Real, DoublePrecision) | (DoublePrecision, Real) => DoublePrecision,

            // Date/Timestamp
            (Date, Timestamp) | (Timestamp, Date) => Timestamp,

            // Boolean with anything else -> Text
            (Boolean, _) | (_, Boolean) => Text,

            // Date/Timestamp with numbers -> Text
            (Date | Timestamp, SmallInt | Integer | BigInt | Real | DoublePrecision) |
            (SmallInt | Integer | BigInt | Real | DoublePrecision, Date | Timestamp) => Text,

            // Catchall for any remaining combinations
            _ => Text,
        }
    }
}

impl fmt::Display for SqlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_null() {
        assert_eq!(SqlType::infer_from_str(""), SqlType::Null);
        assert_eq!(SqlType::infer_from_str("null"), SqlType::Null);
        assert_eq!(SqlType::infer_from_str("NULL"), SqlType::Null);
        assert_eq!(SqlType::infer_from_str("\\N"), SqlType::Null);
    }

    #[test]
    fn test_infer_boolean() {
        assert_eq!(SqlType::infer_from_str("true"), SqlType::Boolean);
        assert_eq!(SqlType::infer_from_str("false"), SqlType::Boolean);
    }

    #[test]
    fn test_infer_integers() {
        assert_eq!(SqlType::infer_from_str("42"), SqlType::SmallInt);
        assert_eq!(SqlType::infer_from_str("32767"), SqlType::SmallInt);
        assert_eq!(SqlType::infer_from_str("32768"), SqlType::Integer);
        assert_eq!(SqlType::infer_from_str("2147483648"), SqlType::BigInt);
    }

    #[test]
    fn test_infer_floats() {
        assert_eq!(SqlType::infer_from_str("3.14"), SqlType::Real);
        assert_eq!(SqlType::infer_from_str("3.14159265359"), SqlType::Real);
    }

    #[test]
    fn test_infer_dates() {
        assert_eq!(SqlType::infer_from_str("2024-01-15"), SqlType::Date);
        assert_eq!(SqlType::infer_from_str("2024/01/15"), SqlType::Date);
    }

    #[test]
    fn test_infer_timestamps() {
        assert_eq!(SqlType::infer_from_str("2024-01-15 10:30:00"), SqlType::Timestamp);
        assert_eq!(SqlType::infer_from_str("2024-01-15T10:30:00"), SqlType::Timestamp);
    }

    #[test]
    fn test_infer_text() {
        assert_eq!(SqlType::infer_from_str("hello world"), SqlType::Text);
        assert_eq!(SqlType::infer_from_str("abc123"), SqlType::Text);
    }

    #[test]
    fn test_type_merge() {
        assert_eq!(SqlType::SmallInt.merge(&SqlType::Integer), SqlType::Integer);
        assert_eq!(SqlType::Integer.merge(&SqlType::BigInt), SqlType::BigInt);
        assert_eq!(SqlType::SmallInt.merge(&SqlType::Real), SqlType::DoublePrecision);
        assert_eq!(SqlType::Integer.merge(&SqlType::Text), SqlType::Text);
        assert_eq!(SqlType::Null.merge(&SqlType::Integer), SqlType::Integer);
    }
}
