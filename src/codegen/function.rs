use crate::codegen::{VMFunction, Value, Visitor};

use crate::parser::Function;

use std::rc::Rc;

impl Visitor {
    pub fn visit_fn(&mut self, f: Function) -> VMFunction {
        self.variables.insert(
            f.name.to_string(),
            Value::Function(
                f.name.clone(),
                VMFunction {
                    decl: Rc::new(f.clone()),
                    call_count:0
                },
            ),
        );
        VMFunction { decl: Rc::new(f), call_count:0 }
    }
}
