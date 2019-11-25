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
use log::*;
use snafu::Snafu;

#[derive(Snafu, Debug, Clone)]
pub enum InterpreterError {
    #[snafu(display("Invalid unary operator {}", operator))]
    InvalidUnaryOperator { operator: Operator },
}

pub struct Interpreter {}

impl Interpreter {
    pub fn exec_ast(ast: &Ast) -> Result<f64, Error> {
        debug!(
            "Starting to execute interpretation engine on AST: {:?}",
            ast
        );

        Interpreter::_exec_ast(ast)
    }

    fn _exec_ast(ast: &Ast) -> Result<f64, Error> {
        match ast {
            Ast::Number(n) => Ok(*n),
            Ast::UnaryOperator { operator, child } => {
                let result = Interpreter::_exec_ast(&child)?;

                match *operator {
                    Operator::Plus => Ok(result),
                    Operator::Minus => Ok(-result),
                    operator => Err(InterpreterError::InvalidUnaryOperator { operator }.into()),
                }
            }
            Ast::BinaryOperator {
                operator,
                left,
                right,
            } => {
                let left = Interpreter::_exec_ast(&left)?;
                let right = Interpreter::_exec_ast(&right)?;

                Ok(match operator {
                    Operator::Plus => left + right,
                    Operator::Minus => left - right,
                    Operator::Multiply => left * right,
                    Operator::Divide => left / right,
                })
            }
            Ast::Parenthesis { child } => Interpreter::_exec_ast(&child),
        }
    }

    pub fn exec(s: &str) -> Result<f64, Error> {
        debug!("Starting to execute interpretation engine on string: {}", s);

        Interpreter::_exec_ast(&AstBuilder::build_ast(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Interpreter;

    #[test]
    fn test_simple_expression() {
        assert_eq!(Interpreter::exec("1 + 2").unwrap() as i32, 3);
        assert_eq!(Interpreter::exec("2 + 2  * 2").unwrap() as i32, 6);
    }
}
