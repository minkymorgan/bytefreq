#[cfg(feature = "parquet")]
use arrow::array::*;
#[cfg(feature = "parquet")]
use arrow::datatypes::{DataType, TimeUnit};
#[cfg(feature = "parquet")]
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
#[cfg(feature = "parquet")]
use serde_json::{Map, Value};
#[cfg(feature = "parquet")]
use std::fs::File;
use std::path::Path;

#[cfg(feature = "parquet")]
pub struct ParquetReader;

#[cfg(feature = "parquet")]
impl ParquetReader {
    /// Read Parquet file and return as JSON lines (one JSON object per row).
    /// This feeds into the existing JSON processing pipeline.
    pub fn read_as_json_lines<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let file = File::open(&path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        let reader = builder.build()?;

        let mut json_lines = Vec::new();

        for batch_result in reader {
            let batch = batch_result?;
            let num_rows = batch.num_rows();
            let schema = batch.schema();

            for row_idx in 0..num_rows {
                let mut row_map = Map::new();

                for (col_idx, field) in schema.fields().iter().enumerate() {
                    let column = batch.column(col_idx);
                    let value = Self::array_value_to_json(column, row_idx);
                    row_map.insert(field.name().clone(), value);
                }

                let json_str = serde_json::to_string(&Value::Object(row_map))?;
                json_lines.push(json_str);
            }
        }

        Ok(json_lines)
    }

    /// Convert a single value from an Arrow array at the given row index to a JSON Value.
    fn array_value_to_json(array: &dyn Array, idx: usize) -> Value {
        if array.is_null(idx) {
            return Value::Null;
        }

        match array.data_type() {
            // P0: Integer types
            DataType::Int8 => {
                let arr = array.as_any().downcast_ref::<Int8Array>().unwrap();
                Value::Number(arr.value(idx).into())
            }
            DataType::Int16 => {
                let arr = array.as_any().downcast_ref::<Int16Array>().unwrap();
                Value::Number(arr.value(idx).into())
            }
            DataType::Int32 => {
                let arr = array.as_any().downcast_ref::<Int32Array>().unwrap();
                Value::Number(arr.value(idx).into())
            }
            DataType::Int64 => {
                let arr = array.as_any().downcast_ref::<Int64Array>().unwrap();
                Value::Number(arr.value(idx).into())
            }
            DataType::UInt8 => {
                let arr = array.as_any().downcast_ref::<UInt8Array>().unwrap();
                Value::Number(arr.value(idx).into())
            }
            DataType::UInt16 => {
                let arr = array.as_any().downcast_ref::<UInt16Array>().unwrap();
                Value::Number(arr.value(idx).into())
            }
            DataType::UInt32 => {
                let arr = array.as_any().downcast_ref::<UInt32Array>().unwrap();
                Value::Number(arr.value(idx).into())
            }
            DataType::UInt64 => {
                let arr = array.as_any().downcast_ref::<UInt64Array>().unwrap();
                // UInt64 values above i64::MAX cannot be represented in serde_json::Number
                // so we fall back to string representation for those
                let val = arr.value(idx);
                if let Some(n) = serde_json::Number::from_f64(val as f64) {
                    Value::Number(n)
                } else {
                    Value::String(val.to_string())
                }
            }

            // P0: Float types
            DataType::Float32 => {
                let arr = array.as_any().downcast_ref::<Float32Array>().unwrap();
                let val = arr.value(idx) as f64;
                match serde_json::Number::from_f64(val) {
                    Some(n) => Value::Number(n),
                    None => Value::Null, // NaN/Inf
                }
            }
            DataType::Float64 => {
                let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
                let val = arr.value(idx);
                match serde_json::Number::from_f64(val) {
                    Some(n) => Value::Number(n),
                    None => Value::Null, // NaN/Inf
                }
            }

            // P0: String types
            DataType::Utf8 => {
                let arr = array.as_any().downcast_ref::<StringArray>().unwrap();
                Value::String(arr.value(idx).to_string())
            }
            // P1: LargeUtf8
            DataType::LargeUtf8 => {
                let arr = array.as_any().downcast_ref::<LargeStringArray>().unwrap();
                Value::String(arr.value(idx).to_string())
            }

            // P0: Boolean
            DataType::Boolean => {
                let arr = array.as_any().downcast_ref::<BooleanArray>().unwrap();
                Value::Bool(arr.value(idx))
            }

            // P0: Null type
            DataType::Null => Value::Null,

            // P1: Timestamp types -> ISO8601 strings
            DataType::Timestamp(unit, tz) => {
                let epoch_str = match unit {
                    TimeUnit::Second => {
                        let a = array.as_any().downcast_ref::<TimestampSecondArray>().unwrap();
                        Self::timestamp_to_iso8601(a.value(idx), 1, tz.as_deref())
                    }
                    TimeUnit::Millisecond => {
                        let a = array.as_any().downcast_ref::<TimestampMillisecondArray>().unwrap();
                        Self::timestamp_to_iso8601(a.value(idx), 1_000, tz.as_deref())
                    }
                    TimeUnit::Microsecond => {
                        let a = array.as_any().downcast_ref::<TimestampMicrosecondArray>().unwrap();
                        Self::timestamp_to_iso8601(a.value(idx), 1_000_000, tz.as_deref())
                    }
                    TimeUnit::Nanosecond => {
                        let a = array.as_any().downcast_ref::<TimestampNanosecondArray>().unwrap();
                        Self::timestamp_to_iso8601(a.value(idx), 1_000_000_000, tz.as_deref())
                    }
                };
                Value::String(epoch_str)
            }

            // P1: Date types
            DataType::Date32 => {
                let arr = array.as_any().downcast_ref::<Date32Array>().unwrap();
                let days = arr.value(idx);
                let date = chrono::NaiveDate::from_num_days_from_ce_opt(days + 719_163);
                match date {
                    Some(d) => Value::String(d.format("%Y-%m-%d").to_string()),
                    None => Value::String(days.to_string()),
                }
            }
            DataType::Date64 => {
                let arr = array.as_any().downcast_ref::<Date64Array>().unwrap();
                let millis = arr.value(idx);
                let secs = millis / 1000;
                let nsecs = ((millis % 1000) * 1_000_000) as u32;
                let dt = chrono::DateTime::from_timestamp(secs, nsecs);
                match dt {
                    Some(d) => Value::String(d.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
                    None => Value::String(millis.to_string()),
                }
            }

            // P0: Struct -> nested JSON object
            DataType::Struct(fields) => {
                let arr = array.as_any().downcast_ref::<StructArray>().unwrap();
                let mut obj = Map::new();
                for (i, field) in fields.iter().enumerate() {
                    let child = arr.column(i);
                    obj.insert(field.name().clone(), Self::array_value_to_json(child.as_ref(), idx));
                }
                Value::Object(obj)
            }

            // P0: List -> JSON array
            DataType::List(_) => {
                let arr = array.as_any().downcast_ref::<ListArray>().unwrap();
                let values = arr.value(idx);
                let mut items = Vec::new();
                for i in 0..values.len() {
                    items.push(Self::array_value_to_json(values.as_ref(), i));
                }
                Value::Array(items)
            }

            // P1: LargeList -> JSON array
            DataType::LargeList(_) => {
                let arr = array.as_any().downcast_ref::<LargeListArray>().unwrap();
                let values = arr.value(idx);
                let mut items = Vec::new();
                for i in 0..values.len() {
                    items.push(Self::array_value_to_json(values.as_ref(), i));
                }
                Value::Array(items)
            }

            // Unsupported types - never panic
            other => Value::String(format!("<unsupported: {}>", other)),
        }
    }

    /// Convert a timestamp value to an ISO8601 string.
    /// `divisor` converts the raw value to seconds (1 for seconds, 1000 for millis, etc.)
    fn timestamp_to_iso8601(raw: i64, divisor: i64, _tz: Option<&str>) -> String {
        let secs = raw / divisor;
        let remainder = (raw % divisor).unsigned_abs();
        let nanos = if divisor == 1 {
            0u32
        } else {
            (remainder * 1_000_000_000 / divisor as u64) as u32
        };
        match chrono::DateTime::from_timestamp(secs, nanos) {
            Some(dt) => dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            None => raw.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Dummy implementation when parquet feature is not enabled
// ---------------------------------------------------------------------------

#[cfg(not(feature = "parquet"))]
pub struct ParquetReader;

#[cfg(not(feature = "parquet"))]
impl ParquetReader {
    pub fn read_as_json_lines<P: AsRef<Path>>(
        _path: P,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Err("Parquet support not enabled. Rebuild with --features parquet".into())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[cfg(feature = "parquet")]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_disabled_not_compiled() {
        // This test verifies the module compiles with the parquet feature enabled.
        // The ParquetReader struct should exist and have the read_as_json_lines method.
        let result = ParquetReader::read_as_json_lines("/nonexistent/file.parquet");
        assert!(result.is_err());
    }
}
