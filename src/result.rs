use std::result::Result;
use error::ErrorKind;

use toml::Value;

/// Type of a Value
pub enum Type {
    String,
    Integer,
    Float,
    Boolean,
    Datetime,
    Array,
    Table,
}

pub enum Error<E> {
    TomlQueryError(::error::Error),
    Other(E),
}

impl<E> From<E> for Error<E> {
    fn from(e: E) -> Error<E> {
        Error::Other(e)
    }
}

impl Type {
    fn matches(&self, v: &Value) -> bool {
        match (self, v) {
            (&Type::String, &Value::String(_))     |
            (&Type::Integer, &Value::Integer(_))   |
            (&Type::Float, &Value::Float(_))       |
            (&Type::Boolean, &Value::Boolean(_))   |
            (&Type::Datetime, &Value::Datetime(_)) |
            (&Type::Array, &Value::Array(_))       |
            (&Type::Table, &Value::Table(_))       => true,
            (_, _)                  => false,
        }
    }

    fn name(&self) -> &'static str {
        match *self {
            Type::Array    => "Array",
            Type::Boolean  => "Boolean",
            Type::Datetime => "Datetime",
            Type::Float    => "Float",
            Type::Integer  => "Integer",
            Type::String   => "String",
            Type::Table    => "Table",
        }
    }
}

pub trait GetResultAsType<E> {
    type Output;
    fn as_type(self, t: Type) -> Result<Self::Output, Error<E>>;
}

impl<E> GetResultAsType<E> for Result<Value, Error<E>> {
    type Output = Value;

    fn as_type(self, t: Type) -> Result<Self::Output, Error<E>> {
        self.and_then(|o| {
            if t.matches(&o) {
                Ok(o)
            } else {
                let e = ErrorKind::TypeError(t.name(), ::util::name_of_val(&o));
                Err(Error::TomlQueryError(e.into()))
            }
        })
    }
}

impl<'a, E> GetResultAsType<E> for Result<&'a Value, Error<E>> {
    type Output = &'a Value;

    fn as_type(self, t: Type) -> Result<Self::Output, Error<E>> {
        self.and_then(|o| {
            if t.matches(&o) {
                Ok(o)
            } else {
                let e = ErrorKind::TypeError(t.name(), ::util::name_of_val(&o));
                Err(Error::TomlQueryError(e.into()))
            }
        })
    }
}


impl<E> GetResultAsType<E> for Result<Option<Value>, Error<E>> {
    type Output = Option<Value>;

    fn as_type(self, t: Type) -> Result<Self::Output, Error<E>> {
        self.and_then(|o| match o {
            None    => Ok(None),
            Some(x) => if t.matches(&x) {
                Ok(Some(x))
            } else {
                let e = ErrorKind::TypeError(t.name(), ::util::name_of_val(&x));
                Err(Error::TomlQueryError(e.into()))
            }
        })
    }
}

impl<'a, E> GetResultAsType<E> for Result<Option<&'a Value>, Error<E>> {
    type Output = Option<&'a Value>;

    fn as_type(self, t: Type) -> Result<Self::Output, Error<E>> {
        self.and_then(|o| match o {
            None    => Ok(None),
            Some(x) => if t.matches(&x) {
                Ok(Some(x))
            } else {
                let e = ErrorKind::TypeError(t.name(), ::util::name_of_val(&x));
                Err(Error::TomlQueryError(e.into()))
            }
        })
    }
}


#[cfg(test)]
mod high_level_fn_test {
    use read::*;
    use read::typed::*;
    use toml::Value;
    use toml::from_str as toml_from_str;
    use super::*;

    #[test]
    fn test_read_table_value() {
        let toml : Value = toml_from_str(r#"
        [table]
        a = 1
        "#).unwrap();

        let val  = toml.read_int(&String::from("table.a"));

        assert!(val.is_ok());
        assert_eq!(val.unwrap(), 1);
    }

    #[test]
    fn test_read_table_value_result_ext() {
        use super::GetResultAsType;
        let toml : Value = toml_from_str(r#"
        [table]
        a = 1
        "#).unwrap();

        let val = toml.read(&String::from("table.a")).as_type(Type::Integer);

        assert!(val.is_ok());
        assert_eq!(*val.unwrap().unwrap(), Value::Integer(1));

        assert!(toml.read(&String::from("table.a")).as_type(Type::String).is_err());
        assert!(toml.read(&String::from("table.a")).as_type(Type::Float).is_err());
        assert!(toml.read(&String::from("table.a")).as_type(Type::Boolean).is_err());
        assert!(toml.read(&String::from("table.a")).as_type(Type::Datetime).is_err());
        assert!(toml.read(&String::from("table.a")).as_type(Type::Array).is_err());
        assert!(toml.read(&String::from("table.a")).as_type(Type::Table).is_err());
    }

}
