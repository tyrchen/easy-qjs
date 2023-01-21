use crate::JsonValue;
use js::{Array, Ctx, FromAtom, FromJs, IntoJs, Null, Object};
use serde::Serialize;
use serde_json::{json, Value};

impl<'js> FromJs<'js> for JsonValue {
    fn from_js(ctx: Ctx<'js>, val: js::Value<'js>) -> Result<Self, js::Error> {
        let v = match val {
            val if val.type_name() == "null" => Value::Null,
            val if val.type_name() == "undefined" => Value::Null,
            val if val.is_bool() => val.as_bool().unwrap().into(),
            val if val.is_string() => match val.into_string().unwrap().to_string() {
                Ok(v) => Value::String(v),
                Err(e) => return Err(e),
            },
            val if val.is_int() => val.as_int().unwrap().into(),
            val if val.is_float() => val.as_float().unwrap().into(),
            val if val.is_array() => {
                let v = val.as_array().unwrap();
                let mut x = Vec::with_capacity(v.len());
                for i in v.iter() {
                    let v = i?;
                    let v = JsonValue::from_js(ctx, v)?;
                    x.push(v.into());
                }
                Value::Array(x)
            }
            val if val.is_object() => {
                // Extract the value as an object
                let v = val.into_object().unwrap();

                // Check to see if this object is a function. We don't support it
                if v.as_function().is_some() {
                    return Ok(Self::null());
                }
                // This object is a normal object
                let mut x = json!({});
                for i in v.props() {
                    let (k, v) = i?;
                    let k = String::from_atom(k)?;
                    let v = JsonValue::from_js(ctx, v)?;
                    x[k] = v.into();
                }
                x.into()
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
                    Ok(js::Value::new_float(ctx, num.as_f64().unwrap()))
                } else if num.is_i64() {
                    Ok(js::Value::new_number(ctx, num.as_i64().unwrap() as _))
                } else {
                    Ok(js::Value::new_number(ctx, num.as_u64().unwrap() as _))
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
