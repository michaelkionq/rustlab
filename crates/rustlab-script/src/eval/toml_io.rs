use std::collections::HashMap;
use num_complex::Complex;
use ndarray::Array1;
use toml::Value as Toml;
use super::value::Value;

// ─── Save ────────────────────────────────────────────────────────────────────

pub fn save_toml(path: &str, value: &Value) -> Result<(), String> {
    let table = match value {
        Value::Struct(_) => value_to_toml(value)?,
        other => return Err(format!(
            "save: TOML requires a struct at the top level, got {}",
            other.type_name()
        )),
    };
    let text = toml::to_string_pretty(&table)
        .map_err(|e| format!("save: TOML serialization failed: {e}"))?;
    std::fs::write(path, text)
        .map_err(|e| format!("save: {e}"))
}

// ─── Load ────────────────────────────────────────────────────────────────────

pub fn load_toml(path: &str) -> Result<Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("load: {e}"))?;
    let table: Toml = text.parse::<Toml>()
        .map_err(|e| format!("load: TOML parse error: {e}"))?;
    Ok(toml_to_value(table))
}

// ─── Value → toml::Value ─────────────────────────────────────────────────────

fn value_to_toml(v: &Value) -> Result<Toml, String> {
    match v {
        Value::Scalar(f) => {
            // Write as integer if it's a whole number in i64 range
            if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                Ok(Toml::Integer(*f as i64))
            } else {
                Ok(Toml::Float(*f))
            }
        }
        Value::Bool(b) => Ok(Toml::Boolean(*b)),
        Value::Str(s) => Ok(Toml::String(s.clone())),
        Value::Vector(cv) => {
            let mut arr = Vec::with_capacity(cv.len());
            for &c in cv.iter() {
                if c.im.abs() > 1e-15 {
                    return Err("save: cannot serialize complex values to TOML".into());
                }
                arr.push(scalar_to_toml(c.re));
            }
            Ok(Toml::Array(arr))
        }
        Value::Matrix(cm) => {
            let (rows, cols) = cm.dim();
            let mut arr = Vec::with_capacity(rows);
            for r in 0..rows {
                let mut row = Vec::with_capacity(cols);
                for c in 0..cols {
                    let v = cm[[r, c]];
                    if v.im.abs() > 1e-15 {
                        return Err("save: cannot serialize complex values to TOML".into());
                    }
                    row.push(scalar_to_toml(v.re));
                }
                arr.push(Toml::Array(row));
            }
            Ok(Toml::Array(arr))
        }
        Value::Struct(fields) => {
            let mut table = toml::map::Map::new();
            // Sort keys for deterministic output
            let mut keys: Vec<&String> = fields.keys().collect();
            keys.sort();
            for key in keys {
                let val = &fields[key];
                table.insert(key.clone(), value_to_toml(val)?);
            }
            Ok(Toml::Table(table))
        }
        Value::Tuple(items) => {
            let mut arr = Vec::with_capacity(items.len());
            for item in items {
                arr.push(value_to_toml(item)?);
            }
            Ok(Toml::Array(arr))
        }
        other => Err(format!(
            "save: cannot serialize {} to TOML",
            other.type_name()
        )),
    }
}

fn scalar_to_toml(f: f64) -> Toml {
    if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
        Toml::Integer(f as i64)
    } else {
        Toml::Float(f)
    }
}

// ─── toml::Value → Value ─────────────────────────────────────────────────────

fn toml_to_value(tv: Toml) -> Value {
    match tv {
        Toml::Table(map) => {
            let fields: HashMap<String, Value> = map.into_iter()
                .map(|(k, v)| (k, toml_to_value(v)))
                .collect();
            Value::Struct(fields)
        }
        Toml::String(s) => Value::Str(s),
        Toml::Integer(i) => Value::Scalar(i as f64),
        Toml::Float(f) => Value::Scalar(f),
        Toml::Boolean(b) => Value::Bool(b),
        Toml::Datetime(dt) => Value::Str(dt.to_string()),
        Toml::Array(arr) => array_to_value(arr),
    }
}

fn array_to_value(arr: Vec<Toml>) -> Value {
    if arr.is_empty() {
        return Value::Vector(Array1::zeros(0));
    }

    // Check if all elements are numeric (Integer or Float)
    let all_numeric = arr.iter().all(|v| matches!(v, Toml::Integer(_) | Toml::Float(_)));
    if all_numeric {
        let cvec: Array1<Complex<f64>> = Array1::from_vec(
            arr.into_iter().map(|v| {
                let f = match v {
                    Toml::Integer(i) => i as f64,
                    Toml::Float(f) => f,
                    _ => unreachable!(),
                };
                Complex::new(f, 0.0)
            }).collect()
        );
        return Value::Vector(cvec);
    }

    // Mixed or non-numeric array → Tuple
    Value::Tuple(arr.into_iter().map(toml_to_value).collect())
}
