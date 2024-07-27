use crate::codegen::{VMFunction, Value, Visitor};

use crate::parser::Function;

use std::sync::Arc;

impl Visitor {
    pub fn visit_fn(&mut self, f: Function) -> VMFunction {
        self.variables.insert(
            f.name.to_string(),
            Value::Function(
                f.name.clone(),
                VMFunction {
                    decl: Arc::new(f.clone()),
                    call_count:0
                },
            ),
        );
        VMFunction { decl: Arc::new(f), call_count:0 }
    }
}
