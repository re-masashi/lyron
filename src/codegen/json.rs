use crate::codegen::{VMError, Value, Visitor};
use serde_json::{from_str as serde_from_str, Value as SerdeValue};
// use gxhash::{HashMap, HashMapExt};
use std::collections::HashMap;


pub fn json_parse(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    if args.is_empty() {
        return Ok(Value::None);
    }
    match serde_from_str::<SerdeValue>(args[0].to_string().as_str()) {
        Ok(result) => Ok(serde_to_value(&result)),
        Err(e) => {
            Err(VMError {
                type_: "InvalidJSONError".to_string(),
                cause: format!(
                    "
    {text}
    {pointy}
        {line}: {pos}
    ",
                    text = args[0].to_string().lines().collect::<Vec<_>>()[e.line() - 1],
                    pointy = ("~".repeat(e.column()) + "^"),
                    line = e.line(),
                    pos = e.column(),
                )
                .to_string(),
            })
        }
    }
}

pub fn json_dumps(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    Ok(Value::Str(value_to_json(args[0].clone())))
}

fn serde_to_value(ser: &SerdeValue) -> Value {
    match ser {
        SerdeValue::Null => Value::None,
        SerdeValue::Bool(b) => Value::Boolean(*b),
        SerdeValue::Number(n) => {
            if n.is_f64() {
                Value::Float64(match n.as_f64() {
                    Some(s) => s,
                    None => unreachable!(),
                })
            } else if n.is_i64() {
                Value::Int64(match n.as_i64() {
                    Some(s) => s,
                    None => unreachable!(),
                })
            } else {
                Value::Int64(match n.as_u64() {
                    Some(s) => s as i64,
                    None => unreachable!(),
                })
            }
        }
        SerdeValue::String(s) => Value::Str(s.to_string()),
        SerdeValue::Array(a) => {
            let mut value_arr: Vec<Value> = Vec::new();
            for val in a {
                value_arr.push(serde_to_value(val));
            }
            Value::Array(value_arr)
        }
        SerdeValue::Object(o) => {
            let mut valhash: HashMap<String, Value> = HashMap::new();
            for (k, v) in o {
                valhash.insert(k.to_string(), serde_to_value(v));
            }
            Value::Dict(valhash)
        }
    }
}

fn value_to_json(value: Value) -> String {
    match value {
        Value::None => "null".to_string(),
        Value::Str(s) => s,
        Value::Array(a) => {
            let mut stringbuf = String::from("[");
            for elem in a {
                stringbuf += &value_to_json(elem);
                stringbuf += ",";
            }
            if stringbuf.len() != 1 {
                // [
                stringbuf.pop(); // ','
            }
            stringbuf += "]";
            stringbuf
        }
        Value::Dict(h) => {
            let mut stringbuf = String::from("{");
            for (k, v) in h {
                stringbuf += &k;
                stringbuf += &value_to_json(v);
                stringbuf += ",";
            }
            if stringbuf.len() != 1 {
                // [
                stringbuf.pop(); // ','
            }
            stringbuf += "]";
            stringbuf
        }
        x => x.to_string(),
    }
}
