use crate::parser::{ExprValue};
use crate::codegen::Visitor;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};
use std::collections::HashMap;
use std::{slice, mem};
use crate::lexer::tokens::TokenType;
use crate::codegen::Value::Function as VMFunBase;
use crate::codegen::uoe;
use crate::codegen::Callable;

type VMValue = crate::codegen::Value;

/// The basic JIT class.
pub struct JIT<'a> {
    /// The function builder context, which is reused across multiple
    /// FunctionBuilder instances.
    builder_context: FunctionBuilderContext,

    /// The main Cranelift context, which holds the state for codegen. Cranelift
    /// separates this from `Module` to allow for parallel compilation, with a
    /// context per thread, though this isn't in the simple demo here.
    ctx: codegen::Context,

    /// The data description, which is to data objects what `ctx` is to functions.
    data_description: DataDescription,

    /// The module, with the jit backend, which manages the JIT'd
    /// functions.
    module: JITModule,
    visitor: &'a mut Visitor,
}



impl<'a> JIT<'a>{
    pub fn new(visitor: &'a mut Visitor,) -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_description: DataDescription::new(),
            module,
            visitor,
        }
    }
    /// Create a zero-initialized data section.
    pub fn create_data(&mut self, name: &str, contents: Vec<u8>) -> Result<&[u8], String> {
        // The steps here are analogous to `compile`, except that data is much
        // simpler than functions.
        self.data_description.define(contents.into_boxed_slice());
        let id = self
            .module
            .declare_data(name, Linkage::Export, true, false)
            .map_err(|e| e.to_string())?;

        self.module
            .define_data(id, &self.data_description)
            .map_err(|e| e.to_string())?;
        self.data_description.clear();
        self.module.finalize_definitions().unwrap();
        let buffer = self.module.get_finalized_data(id);
        // TODO: Can we move the unsafe into cranelift?
        Ok(unsafe { slice::from_raw_parts(buffer.0, buffer.1) })
    }

    // Translate from toy-language AST nodes into Cranelift IR.
    fn translate(
        &mut self,
        params: Vec<String>,
        the_return: String,
        stmts: Vec<ExprValue>,
    ) -> Result<(), String> {
        // Our toy language currently only supports I64 values, though Cranelift
        // supports other types.
        let int = self.module.target_config().pointer_type();

        for _p in &params {
            self.ctx.func.signature.params.push(AbiParam::new(int));
        }

        // Our toy language currently only supports one return value, though
        // Cranelift is designed to support more.
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        // Create the builder to build a function.
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Create the entry block, to start emitting code in.
        let entry_block = builder.create_block();

        // Since this is the entry block, add block parameters corresponding to
        // the function's parameters.
        //
        // TODO: Streamline the API here.
        builder.append_block_params_for_function_params(entry_block);

        // Tell the builder to emit code in this block.
        builder.switch_to_block(entry_block);

        // And, tell the builder that this block will have no further
        // predecessors. Since it's the entry block, it won't have any
        // predecessors.
        builder.seal_block(entry_block);

        // The toy language allows variables to be declared implicitly.
        // Walk the AST and declare all implicitly-declared variables.
        let variables =
            HashMap::new();

        // Now translate the statements of the function body.
        let mut trans = FunctionTranslator {
            int,
            builder,
            variables,
            module: &mut self.module,
            visitor:self.visitor,
        };
        let mut return_value: Value = match Value::with_number(0){
            Some(m)=>m,
            _=>unreachable!()
        };
        for expr in stmts {
            return_value = trans.translate_expr(expr);
        }

        // Emit the return instruction.
        trans.builder.ins().return_(&[return_value]);

        // Tell the builder we're done with this function.
        trans.builder.finalize();
        Ok(())
    }

    pub fn compile(
        &mut self, 
        params: Vec<String>,
        the_return: String,
        stmts: Vec<ExprValue>,
    ) -> Result<*const u8, String> {

        // Then, translate the AST nodes into Cranelift IR.
        self.translate(params, the_return, stmts)?;

        // Next, declare the function to jit. Functions must be declared
        // before they can be called, or defined.
        //
        // TODO: This may be an area where the API should be streamlined; should
        // we have a version of `declare_function` that automatically declares
        // the function?
        let id = self
            .module
            .declare_function("", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        // Define the function to jit. This finishes compilation, although
        // there may be outstanding relocations to perform. Currently, jit
        // cannot finish relocations until all functions to be called are
        // defined. For this toy demo for now, we'll just finalize the
        // function below.
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        // Now that compilation is finished, we can clear out the context state.
        self.module.clear_context(&mut self.ctx);

        // Finalize the functions which we just defined, which resolves any
        // outstanding relocations (patching in addresses, now that they're
        // available).
        self.module.finalize_definitions().unwrap();

        // We can now retrieve a pointer to the machine code.
        let code = self.module.get_finalized_function(id);

        Ok(code)
    }
}

/// A collection of state used for translating from toy-language AST nodes
/// into Cranelift IR.
struct FunctionTranslator<'a> {
    int: types::Type,
    builder: FunctionBuilder<'a>,
    variables: HashMap<String, Value>,
    module: &'a mut JITModule,
    visitor: &'a mut Visitor
}

impl<'a> FunctionTranslator<'a> {
    /// When you write out instructions in Cranelift, you get back `Value`s. You
    /// can then use these references in other instructions.
    fn translate_expr(&mut self, expr: ExprValue) -> Value {
        match expr {
            ExprValue::Integer(i) => {
                self.builder.ins().iconst(self.int, i64::from(i))
            }
            ExprValue::BinOp(lhs, t, rhs)=>{
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                match *t {
                    TokenType::Plus=>self.builder.ins().iadd(lhs, rhs),
                    TokenType::Minus=>self.builder.ins().isub(lhs, rhs),
                    TokenType::Mul=>self.builder.ins().imul(lhs,rhs),
                    TokenType::Div=>self.builder.ins().udiv(lhs,rhs),
                    TokenType::Equal=>self.translate_icmp(IntCC::Equal, lhs, rhs),
                    TokenType::NotEq=>self.translate_icmp(IntCC::NotEqual, lhs, rhs),
                    TokenType::Less=>self.translate_icmp(IntCC::SignedLessThan, lhs, rhs),
                    TokenType::LessEq=>self.translate_icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                    TokenType::Greater=>self.translate_icmp(IntCC::SignedGreaterThan, lhs, rhs),
                    TokenType::GreaterEq=>self.translate_icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
                    _=>todo!()
                }
            }
            ExprValue::FnCall(name, args)=>self.translate_call(name, args),
            // ExprValue::BinOp(lhs, TokenType::Minus, rhs)=>{
            //         let lhs = self.translate_expr(*lhs);
            //         let rhs = self.translate_expr(*rhs);
            //         self.builder.ins().isub(lhs, rhs)
            // }
            // ExprValue::BinOp(lhs, TokenType::Div, rhs)=>{
            //         let lhs = self.translate_expr(*lhs);
            //         let rhs = self.translate_expr(*rhs);
            //         self.builder.ins().udiv(lhs, rhs)
            // }
            // ExprValue::BinOp(lhs, TokenType::Mul, rhs)=>{
            //         let lhs = self.translate_expr(*lhs);
            //         let rhs = self.translate_expr(*rhs);
            //         self.builder.ins().imul(lhs, rhs)
            // }
            // ExprValue::BinOp(lhs, TokenType::Equal, rhs)=>self.translate_icmp(IntCC::Equal, *lhs, *rhs),
            // ExprValue::BinOp(lhs, TokenType::NotEq, rhs)=>self.translate_icmp(IntCC::NotEqual, *lhs, *rhs),
            // ExprValue::BinOp(lhs, TokenType::Less, rhs)=>self.translate_icmp(IntCC::SignedLessThan, *lhs, *rhs),
            // ExprValue::BinOp(lhs, Box::<TokenType::Equal>, rhs)=>self.translate_icmp(IntCC::Equal, *lhs, *rhs),


            // Expr::Add(lhs, rhs) => {
            //     let lhs = self.translate_expr(*lhs);
            //     let rhs = self.translate_expr(*rhs);
            //     self.builder.ins().iadd(lhs, rhs)
            // }

            // Expr::Sub(lhs, rhs) => {
            //     let lhs = self.translate_expr(*lhs);
            //     let rhs = self.translate_expr(*rhs);
            //     self.builder.ins().isub(lhs, rhs)
            // }

            // Expr::Mul(lhs, rhs) => {
            //     let lhs = self.translate_expr(*lhs);
            //     let rhs = self.translate_expr(*rhs);
            //     self.builder.ins().imul(lhs, rhs)
            // }

            // Expr::Div(lhs, rhs) => {
            //     let lhs = self.translate_expr(*lhs);
            //     let rhs = self.translate_expr(*rhs);
            //     self.builder.ins().udiv(lhs, rhs)
            // }

            // Expr::Eq(lhs, rhs) => self.translate_icmp(IntCC::Equal, *lhs, *rhs),
            // Expr::Ne(lhs, rhs) => self.translate_icmp(IntCC::NotEqual, *lhs, *rhs),
            // Expr::Lt(lhs, rhs) => self.translate_icmp(IntCC::SignedLessThan, *lhs, *rhs),
            // Expr::Le(lhs, rhs) => self.translate_icmp(IntCC::SignedLessThanOrEqual, *lhs, *rhs),
            // Expr::Gt(lhs, rhs) => self.translate_icmp(IntCC::SignedGreaterThan, *lhs, *rhs),
            // Expr::Ge(lhs, rhs) => self.translate_icmp(IntCC::SignedGreaterThanOrEqual, *lhs, *rhs),
            // Expr::Call(name, args) => self.translate_call(name, args),
            // Expr::GlobalDataAddr(name) => self.translate_global_data_addr(name),
            ExprValue::Identifier(name) => {
                // `use_var` is used to read the value of a variable.
                *self.variables.get(&name).expect(format!("variable `{}` not defined", name).as_str())
            }
            // Expr::Assign(name, expr) => self.translate_assign(name, *expr),
            // Expr::IfElse(condition, then_body, else_body) => {
            //     self.translate_if_else(*condition, then_body, else_body)
            // }
            ExprValue::IfElse{cond, if_,else_}=>self.translate_if_else(*cond, if_, else_),
            ExprValue::While(condition, loop_body) =>self.translate_while_loop(*condition, loop_body),
            ExprValue::Assign{name, value}=>self.translate_assign(name, *value),
            x=>panic!("not done {:#?}", x)
        }
    }

    fn translate_assign(&mut self, name: String, expr: ExprValue) -> Value {
        // `def_var` is used to write the value of a variable. Note that
        // variables can have multiple definitions. Cranelift will
        // convert them into SSA form for itself automatically.
        let new_value = self.translate_expr(expr);
        let variable = self.variables.insert(name, new_value);
        new_value
    }

    fn translate_icmp(&mut self, cmp: IntCC, lhs: Value, rhs: Value) -> Value {
        self.builder.ins().icmp(cmp, lhs, rhs)
    }

    fn translate_if_else(
        &mut self,
        condition: ExprValue,
        then_body: Vec<ExprValue>,
        else_body: Vec<ExprValue>,
    ) -> Value {
        let condition_value = self.translate_expr(condition);

        let then_block = self.builder.create_block();
        let else_block = self.builder.create_block();
        let merge_block = self.builder.create_block();

        // If-else constructs in the toy language have a return value.
        // In traditional SSA form, this would produce a PHI between
        // the then and else bodies. Cranelift uses block parameters,
        // so set up a parameter in the merge block, and we'll pass
        // the return values to it from the branches.
        self.builder.append_block_param(merge_block, self.int);

        // Test the if condition and conditionally branch.
        self.builder
            .ins()
            .brif(condition_value, then_block, &[], else_block, &[]);

        self.builder.switch_to_block(then_block);
        self.builder.seal_block(then_block);
        let mut then_return = self.builder.ins().iconst(self.int, 0);
        for expr in then_body {
            then_return = self.translate_expr(expr);
        }

        // Jump to the merge block, passing it the block return value.
        self.builder.ins().jump(merge_block, &[then_return]);

        self.builder.switch_to_block(else_block);
        self.builder.seal_block(else_block);
        let mut else_return = self.builder.ins().iconst(self.int, 0);
        for expr in else_body {
            else_return = self.translate_expr(expr);
        }

        // Jump to the merge block, passing it the block return value.
        self.builder.ins().jump(merge_block, &[else_return]);

        // Switch to the merge block for subsequent statements.
        self.builder.switch_to_block(merge_block);

        // We've now seen all the predecessors of the merge block.
        self.builder.seal_block(merge_block);

        // Read the value of the if-else by reading the merge block
        // parameter.
        let phi = self.builder.block_params(merge_block)[0];

        phi
    }

    fn translate_while_loop(&mut self, condition: ExprValue, loop_body: Vec<ExprValue>) -> Value {
        let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();

        self.builder.ins().jump(header_block, &[]);
        self.builder.switch_to_block(header_block);

        let condition_value = self.translate_expr(condition);
        self.builder
            .ins()
            .brif(condition_value, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        self.builder.seal_block(body_block);

        for expr in loop_body {
            self.translate_expr(expr);
        }
        self.builder.ins().jump(header_block, &[]);

        self.builder.switch_to_block(exit_block);

        // We've reached the bottom of the loop, so there will be no
        // more backedges to the header to exits to the bottom.
        self.builder.seal_block(header_block);
        self.builder.seal_block(exit_block);

        // Just return 0 for now.
        self.builder.ins().iconst(self.int, 0)
    }

    fn translate_call(&mut self, name: String, args: Vec<ExprValue>) -> Value {
        todo!()
    }

    fn translate_global_data_addr(&mut self, name: String) -> Value {
        let sym = self
            .module
            .declare_data(&name, Linkage::Export, true, false)
            .expect("problem declaring data object");
        let local_id = self.module.declare_data_in_func(sym, self.builder.func);

        let pointer = self.module.target_config().pointer_type();
        self.builder.ins().symbol_value(pointer, local_id)
    }

    fn vm_to_jit(&mut self,val: VMValue)->Value{
        match val {
            VMValue::Int64(i)=>self.builder.ins().iconst(self.int,i as i64),
            VMValue::Float32(f)=>self.builder.ins().f32const(f),
            VMValue::Float64(f)=>self.builder.ins().f64const(f),
            _=>todo!()
        }
    }
}

fn declare_variables(
    int: types::Type,
    builder: &mut FunctionBuilder,
    params: &[String],
    the_return: &str,
    stmts: &[ExprValue],
    entry_block: Block,
) -> HashMap<String, Variable> {
    let mut variables = HashMap::new();
    let mut index = 0;

    for (i, name) in params.iter().enumerate() {
        // TODO: cranelift_frontend should really have an API to make it easy to set
        // up param variables.
        let val = builder.block_params(entry_block)[i];
        let var = declare_variable(int, builder, &mut variables, &mut index, name);
        builder.def_var(var, val);
    }
    let zero = builder.ins().iconst(int, 0);
    let return_variable = declare_variable(int, builder, &mut variables, &mut index, the_return);
    builder.def_var(return_variable, zero);
    for expr in stmts {
        declare_variables_in_stmt(int, builder, &mut variables, &mut index, expr);
    }

    variables
}

/// Recursively descend through the AST, translating all implicit
/// variable declarations.
fn declare_variables_in_stmt(
    int: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    expr: &ExprValue,
) {
    match *expr {
        // ExprValue::Assign(ref name, _) => {
        //     declare_variable(int, builder, variables, index, name);
        // }
        // ExprValue::IfElse(ref _condition, ref then_body, ref else_body) => {
        //     for stmt in then_body {
        //         declare_variables_in_stmt(int, builder, variables, index, stmt);
        //     }
        //     for stmt in else_body {
        //         declare_variables_in_stmt(int, builder, variables, index, stmt);
        //     }
        // }
        // ExprValue::While(ref _condition, ref loop_body) => {
        //     for stmt in loop_body {
        //         declare_variables_in_stmt(int, builder, variables, index, stmt);
        //     }
        // }
        _ => (),
    }
}

/// Declare a single variable declaration.
fn declare_variable(
    int: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    name: &str,
) -> Variable {
    let var = Variable::new(*index);
    if !variables.contains_key(name) {
        variables.insert(name.into(), var);
        builder.declare_var(var, int);
        *index += 1;
    }
    var
}

pub unsafe fn run_code<I, O>(jit: &mut JIT, code:Vec<ExprValue>, input: I) -> Result<O, String> {
    // Pass the string to the JIT, and it returns a raw pointer to machine code.
    let code_ptr = jit.compile(vec![],"".to_string(),code)?;
    // Cast the raw pointer to a typed function pointer. This is unsafe, because
    // this is the critical point where you have to trust that the generated code
    // is safe to be called.
    let code_fn = mem::transmute::<_, fn(I) -> O>(code_ptr);
    // And now we can call it!
    Ok(code_fn(input))
}

pub fn run_foo(jit: &mut JIT, code:Vec<ExprValue>) -> Result<isize, String> {
    unsafe { run_code(jit, code, (1, 0)) }
}
