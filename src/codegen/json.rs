use serde_json::{Value as SerdeValue,from_str as serde_from_str};
use crate::codegen::{Visitor, Value};

pub fn json_parse(args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    if args.is_empty() {
    	return Value::None
    }

    println!("{:#?}", serde_from_str::<SerdeValue>(args[0].to_string().as_str()));
    Value::None
}