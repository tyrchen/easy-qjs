use std::fmt;

use crate::JsonValue;
use itertools::Itertools;
use js::{Array, Ctx, FromAtom, FromJs, IntoJs, Null, Object};
use serde::Serialize;
use serde_json::{json, Value};

impl<'js> FromJs<'js> for JsonValue {
    fn from_js(_ctx: Ctx<'js>, val: js::Value<'js>) -> Result<Self, js::Error> {
        let v = match val {
            val if val.type_name() == "null" => Value::Null,
            val if val.type_name() == "undefined" => Value::Null,
            val if val.is_bool() => val.as_bool().expect("checked bool").into(),
            val if val.is_string() => {
                match val.into_string().expect("checked string").to_string() {
                    Ok(v) => Value::String(v),
                    Err(e) => return Err(e),
                }
            }
            val if val.is_int() => val.as_int().expect("checked int").into(),
            val if val.is_float() => val.as_float().expect("checked float").into(),
            val if val.is_array() => {
                let v = val.as_array().expect("checked array");
                let mut x = Vec::with_capacity(v.len());
                for i in v.iter() {
                    let v = i?;
                    let v = JsonValue::from_js(_ctx, v)?;
                    x.push(v.into());
                }
                Value::Array(x)
            }
            val if val.is_object() => {
                // Extract the value as an object
                let v = val.into_object().expect("checked object");

                // Check to see if this object is a function. We don't support it
                if v.as_function().is_some() {
                    return Ok(Self::null());
                }
                // This object is a normal object
                let mut x = json!({});
                for i in v.props() {
                    let (k, v) = i?;
                    let k = String::from_atom(k)?;
                    let v = JsonValue::from_js(_ctx, v)?;
                    x[k] = v.into();
                }
                x
            }
            _ => Value::Null,
        };
        Ok(v.into())
    }
}

impl<'js> IntoJs<'js> for JsonValue {
    fn into_js(self, ctx: Ctx<'js>) -> Result<js::Value<'js>, js::Error> {
        match self.0 {
            Value::Null => Null.into_js(ctx),
            Value::Bool(v) => Ok(js::Value::new_bool(ctx, v)),
            Value::Number(num) => {
                if num.is_f64() {
                    Ok(js::Value::new_float(
                        ctx,
                        num.as_f64().expect("checked f64"),
                    ))
                } else if num.is_i64() {
                    Ok(js::Value::new_number(
                        ctx,
                        num.as_i64().expect("checked f64") as _,
                    ))
                } else {
                    Ok(js::Value::new_number(
                        ctx,
                        num.as_u64().expect("checked f64") as _,
                    ))
                }
            }
            Value::String(v) => js::String::from_str(ctx, &v)?.into_js(ctx),
            Value::Array(v) => {
                let x = Array::new(ctx)?;
                for (i, v) in v.into_iter().enumerate() {
                    x.set(i, JsonValue(v).into_js(ctx)?)?;
                }
                x.into_js(ctx)
            }
            Value::Object(v) => {
                let x = Object::new(ctx)?;
                for (k, v) in v.into_iter() {
                    x.set(k, JsonValue(v).into_js(ctx)?)?;
                }
                x.into_js(ctx)
            }
        }
    }
}

impl From<Value> for JsonValue {
    fn from(v: Value) -> Self {
        Self(v)
    }
}

impl From<JsonValue> for Value {
    fn from(v: JsonValue) -> Self {
        v.0
    }
}

impl Default for JsonValue {
    fn default() -> Self {
        Self(json!({}))
    }
}

impl JsonValue {
    pub fn null() -> Self {
        Self(Value::Null)
    }

    pub fn array<T: Serialize>(arr: Vec<T>) -> Self {
        Self(json!(arr))
    }

    pub fn object<T: Serialize>(obj: T) -> Self {
        Self(json!(obj))
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Value::Null => write!(f, "null"),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Array(v) => {
                write!(f, "[")?;
                write!(f, "{}", v.iter().join(", "))?;
                write!(f, "]")
            }
            Value::Object(_) => {
                let s = serde_json::to_string_pretty(&self.0).map_err(|_| fmt::Error)?;
                write!(f, "{}", s)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "console")]
    use crate::builtins::con::Console;
    use crate::JsEngine;
    use anyhow::Result;
    use js::Function;

    #[js::bind(object, public)]
    #[quickjs(bare, rename = "print")]
    #[allow(unused_variables)]
    fn print(s: String) {
        println!("{}", s);
    }

    #[tokio::test]
    async fn json_value_should_be_converted_to_js() -> Result<()> {
        let engine = JsEngine::new(vec![])?;
        let _ret: Result<()> = engine.context.with(|ctx| {
            let v = JsonValue::object(json!({
              "name": "John",
              "age": 30,
              "cars": [
                "Ford",
                "BMW",
                "Fiat"
              ]
            }));
            let js = v.clone().into_js(ctx)?;
            assert_eq!(js.type_name(), "object");
            let v1 = JsonValue::from_js(ctx, js)?;
            assert_eq!(v, v1);
            Ok(())
        });
        Ok(())
    }

    #[tokio::test]
    async fn js_object_might_be_converted_as_null() -> Result<()> {
        let engine = JsEngine::new(vec![])?;
        let _ret: Result<()> = engine.context.with(|ctx| {
            let obj = Object::new(ctx)?;
            obj.set("name", "John")?;
            #[cfg(feature = "console")]
            obj.set("obj", Console)?;
            obj.set("fun", Function::new(ctx, print))?;
            let js = obj.into_js(ctx)?;
            let v = JsonValue::from_js(ctx, js)?;
            #[cfg(feature = "console")]
            assert_eq!(v.0, json!({ "name": "John", "obj": {}, "fun": null }));
            #[cfg(not(feature = "console"))]
            assert_eq!(v.0, json!({ "name": "John", "fun": null }));
            Ok(())
        });
        Ok(())
    }
}
