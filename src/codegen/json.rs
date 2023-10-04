use serde_json::{Value as SerdeValue,from_str as serde_from_str,};
use crate::codegen::{Visitor, Value, VMError};
use std::collections::HashMap;

pub fn json_parse(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    if args.is_empty() {
    	return Ok(Value::None)
    }
    match serde_from_str::<SerdeValue>(args[0].to_string().as_str()) {
        Ok(result)=>println!("{:?}", serde_to_value(&result)),
        Err(e)=>{
            return Err(VMError{type_:"InvalidJSONError".to_string(), cause: format!(
"
{text}
{pointy}
    {line}: {pos}
",
                        text=args[0].to_string()
                            .lines()
                            .collect::<Vec<_>>()[(e.line() - 1) as usize],
                        pointy=("~".repeat(e.column()) + "^"),
                        line = e.line(),
                        pos=e.column(),
                    )
                    .to_string()});
        }
    }

    Ok(Value::None)
}

fn serde_to_value(ser: &SerdeValue)->Value{
    match ser {
        SerdeValue::Null=>Value::None,
        SerdeValue::Bool(b)=>Value::Boolean(*b),
        SerdeValue::Number(n)=>todo!(),
        SerdeValue::String(s)=>Value::Str(s.to_string()),
        SerdeValue::Array(a)=>{
            let mut value_arr: Vec<Value> = Vec::new();
            for val in a {
                value_arr.push(serde_to_value(val));
            }
            Value::Array(value_arr)
        },
        SerdeValue::Object(o)=>{
            let mut valhash: HashMap<String, Value> = HashMap::new();
            for (k, v) in o {
                println!("{:#}:{:#}", k,v);
                valhash.insert(k.to_string(), serde_to_value(v));
            }
            Value::Dict(valhash)
        },
    }
}