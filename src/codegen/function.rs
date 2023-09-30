use crate::codegen::{VMFunction, Value, Visitor};

use crate::parser::Function;

use std::rc::Rc;

impl Visitor {
    pub fn visit_fn(&mut self, f: Function) -> VMFunction {
        self.variables.insert(
            f.name.clone(),
            Value::Function(
                f.name.clone(),
                VMFunction {
                    decl: Rc::new(f.clone()),
                },
            ),
        );
        VMFunction { decl: Rc::new(f) }
    }
}
