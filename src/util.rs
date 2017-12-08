use std::collections::BTreeMap;

use toml::Value;

use error::Result;
use error::ErrorKind as EK;

pub fn name_of_val(val: &Value) -> &'static str {
    match *val {
        Value::Array(_)    => "Array",
        Value::Boolean(_)  => "Boolean",
        Value::Datetime(_) => "Datetime",
        Value::Float(_)    => "Float",
        Value::Integer(_)  => "Integer",
        Value::String(_)   => "String",
        Value::Table(_)    => "Table",
    }
}


pub trait FromValue {
    fn from_value(v: Value) -> Result<Self>;
}

impl FromValue for Value {
    fn from_value(v: Value) -> Result<Self> {
        v
    }
}

impl FromValue for bool {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Boolean(b) => Ok(b),
            _                 => Err(EK::TypeError("Boolean", name_of_val(&v))),
        }
    }
}

impl FromValue for f64 {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Float(f) => Ok(f),
            _               => Err(EK::TypeError("Float", name_of_val(&v))),
        }
    }
}

impl FromValue for i64 {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Integer(i) => Ok(i),
            _                 => Err(EK::TypeError("Integer", name_of_val(&v))),
        }
    }
}

impl FromValue for String {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::String(s) => Ok(s),
            _                => Err(EK::TypeError("String", name_of_val(&v))),
        }
    }
}

impl<T> FromValue for Vec<T>
    where T: FromValue
{
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Array(a) => a
                .into_iter()
                .map(FromValue::from_value)
                .fold(Ok(vec![]), |acc, e| {
                    acc.and_then(|a| {
                        a.push(FromValue::from_value(e)?);
                        Ok(a)
                    })
                }),
            _ => Err(EK::TypeError("Array", name_of_val(&v))),
        }
    }
}

impl<T> FromValue for BTreeMap<String, T>
    where T: FromValue
{
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Table(t) => t
                .into_iter()
                .map(|k, v| (k, FromValue::from_value(v)))
                .fold(Ok(BTreeMap::new()), |acc, (k, v)| {
                    acc.and_then(|a| {
                        a.insert(k, FromValue::from_value(v)?);
                        Ok(a)
                    })
                }),
            _ => Err(EK::TypeError("Table", name_of_val(&v))),
        }
    }
}

