use crate::codegen::{VMFunction, Value, Visitor};

use crate::parser::Class;

// use gxhash::{HashMap, HashMapExt};
use std::collections::HashMap;

impl Visitor {
    pub fn visit_class(&self, c: Class) {
        let mut fns: HashMap<String, VMFunction> = HashMap::new();
        for fun in c.fns {
            let (f, _) = fun;
            fns.insert(f.name.to_string(), self.visit_fn(f));
        }
        self.variables
            .borrow_mut()
            .insert(c.name.to_string(), Value::Class(c.name, fns, HashMap::new()));
    }
}
