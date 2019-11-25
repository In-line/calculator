/*
 * Calculator
 * Copyright (c) 2019 Alik Aslanyan <cplusplus256@gmail.com>
 *
 *
 *    This file is part of Calculator.
 *
 *    Calculator is free software; you can redistribute it and/or modify it
 *    under the terms of the GNU General Public License as published by the
 *    Free Software Foundation; either version 3 of the License, or (at
 *    your option) any later version.
 *
 *    This program is distributed in the hope that it will be useful, but
 *    WITHOUT ANY WARRANTY; without even the implied warranty of
 *    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 *    General Public License for more details.
 *
 *    You should have received a copy of the GNU General Public License
 *    along with this program; if not, write to the Free Software Foundation,
 *    Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 */

use crate::ast::{Ast, AstBuilder};
use crate::errors::Error;
use crate::parser::Operator;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "llvm_jit")] {
        use inkwell::{
            builder::Builder, context::Context, execution_engine::ExecutionEngine,
            execution_engine::JitFunction, types::FloatType, values::FloatValue, OptimizationLevel,
        };
        use derive_more::Constructor;
    } else if #[cfg(feature = "cranelift_jit")] {
        use cranelift::prelude::*;
        use cranelift_module::{Linkage, Module};
        use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};
    }
}

use log::*;
use snafu::Snafu;

#[derive(Clone, Copy, Debug)]
pub enum JitOptimizationLevel {
    None,
    Less,
    Default,
    Aggressive,
}

#[cfg(feature = "llvm_jit")]
impl Into<OptimizationLevel> for JitOptimizationLevel {
    fn into(self) -> OptimizationLevel {
        match self {
            JitOptimizationLevel::None => OptimizationLevel::None,
            JitOptimizationLevel::Less => OptimizationLevel::Less,
            JitOptimizationLevel::Default => OptimizationLevel::Default,
            JitOptimizationLevel::Aggressive => OptimizationLevel::Aggressive,
        }
    }
}

impl Default for JitOptimizationLevel {
    fn default() -> Self {
        JitOptimizationLevel::Default
    }
}

#[derive(Snafu, Debug)]
pub enum JitError {
    #[snafu(display("JIT engine doesn't support unary operator: {}", operator))]
    UnsupportedUnaryOperator { operator: Operator },
}

pub struct Jit {}

type JitFunc = unsafe extern "C" fn() -> f64;

impl Jit {
    pub fn exec(s: &str, optimization_level: JitOptimizationLevel) -> Result<f64, Error> {
        debug!("Starting to execute JIT engine on string: {}", s);

        Jit::exec_ast(&AstBuilder::build_ast(s)?, optimization_level)
    }

    #[cfg(feature = "llvm_jit")]
    pub fn exec_ast(ast: &Ast, optimization_level: JitOptimizationLevel) -> Result<f64, Error> {
        debug!("Starting to execute JIT engine on AST: {:?}", ast);

        ExecutionEngine::link_in_mc_jit();

        let context = Context::create();
        let module = context.create_module("calculator");

        let builder = context.create_builder();

        let execution_engine = module
            .create_jit_execution_engine(optimization_level.into())
            .unwrap();

        let f64_type = context.f64_type();
        let fn_type = f64_type.fn_type(&[], false);

        let function = module.add_function("exec", fn_type, None);
        let basic_block = context.append_basic_block(function.clone(), "entry");

        builder.position_at_end(&basic_block);

        #[derive(Constructor)]
        struct RecursiveBuilder<'a> {
            f64_type: FloatType<'a>,
            builder: &'a Builder<'a>,
        }

        impl<'a> RecursiveBuilder<'a> {
            pub fn build(&self, ast: &Ast) -> FloatValue {
                match ast {
                    Ast::Number(n) => self.f64_type.const_float(*n),
                    Ast::UnaryOperator { operator, child } => {
                        let child = self.build(&child);
                        match operator {
                            Operator::Minus => child.const_neg(),
                            Operator::Plus => child,
                            _ => unreachable!(),
                        }
                    }
                    Ast::BinaryOperator {
                        operator,
                        left,
                        right,
                    } => {
                        let left = self.build(&left);
                        let right = self.build(&right);

                        match operator {
                            Operator::Plus => {
                                self.builder.build_float_add(left, right, "plus_temp")
                            }
                            Operator::Minus => {
                                self.builder.build_float_sub(left, right, "minus_temp")
                            }
                            Operator::Divide => {
                                self.builder.build_float_div(left, right, "divide_temp")
                            }
                            Operator::Multiply => {
                                self.builder.build_float_mul(left, right, "multiply_temp")
                            }
                        }
                    }
                    Ast::Parenthesis { child } => self.build(&child),
                }
            }
        }

        let recursive_builder = RecursiveBuilder::new(f64_type, &builder);
        let return_value = recursive_builder.build(ast);
        builder.build_return(Some(&return_value));

        debug!(
            "Generated LLVM IR: {}",
            function.print_to_string().to_string()
        );

        unsafe {
            let jit_function: JitFunction<JitFunc> = execution_engine.get_function("exec").unwrap();

            Ok(jit_function.call())
        }
    }

    #[cfg(feature = "cranelift_jit")]
    pub fn exec_ast(ast: &Ast, _: JitOptimizationLevel) -> Result<f64, Error> {
        let builder = SimpleJITBuilder::new(cranelift_module::default_libcall_names());
        let mut builder_context = FunctionBuilderContext::new();
        let mut module: Module<SimpleJITBackend> = Module::new(builder);
        let mut context = module.make_context();

        context
            .func
            .signature
            .returns
            .push(AbiParam::new(types::F64));

        let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let entry_ebb = builder.create_ebb();

        builder.switch_to_block(entry_ebb);
        builder.seal_block(entry_ebb);

        fn build(builder: &mut FunctionBuilder<'_>, ast: &Ast) -> Value {
            match ast {
                Ast::Number(n) => builder.ins().f64const(*n),
                Ast::UnaryOperator { operator, child } => {
                    let child = build(builder, &child);
                    match operator {
                        Operator::Minus => builder.ins().fneg(child),
                        Operator::Plus => child,
                        _ => unreachable!(),
                    }
                }
                Ast::BinaryOperator {
                    operator,
                    left,
                    right,
                } => {
                    let left = build(builder, &left);
                    let right = build(builder, &right);

                    match operator {
                        Operator::Plus => builder.ins().fadd(left, right),
                        Operator::Minus => builder.ins().fsub(left, right),
                        Operator::Divide => builder.ins().fdiv(left, right),
                        Operator::Multiply => builder.ins().fmul(left, right),
                    }
                }
                Ast::Parenthesis { child } => build(builder, &child),
            }
        }
        let return_value = build(&mut builder, ast);

        let return_variable = Variable::new(0);
        builder.declare_var(return_variable, types::F64);
        builder.def_var(return_variable, return_value);

        let return_var = [builder.use_var(return_variable)];
        let _ = builder.ins().return_(&return_var);

        builder.finalize();

        let function_id = module
            .declare_function("exec", Linkage::Export, &context.func.signature)
            .unwrap();
        module.define_function(function_id, &mut context).unwrap();
        module.clear_context(&mut context);
        module.finalize_definitions();

        unsafe {
            let function: JitFunc = std::mem::transmute(module.get_finalized_function(function_id));
            Ok((function)())
        }
    }
}
