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

use super::interpret::Interpreter;
use super::jit::Jit;
pub use super::jit::JitOptimizationLevel;
use crate::ast::{Ast, AstBuilder};
use crate::errors::Error;
use clone_all::clone_all;
use crossbeam::channel::{bounded, select};
use log::*;
use std::sync::Arc;
use std::thread;

pub struct Hybrid {}

impl Hybrid {}

impl Hybrid {
    pub fn exec(s: &str, optimization_level: JitOptimizationLevel) -> Result<f64, Error> {
        Hybrid::exec_ast(AstBuilder::build_ast(s)?, optimization_level)
    }

    pub fn exec_ast(ast: Ast, optimization_level: JitOptimizationLevel) -> Result<f64, Error> {
        debug!("Starting to execute hybrid engine on AST");

        let current_time = time::precise_time_s();

        let ast = Arc::new(ast);

        let (first_send, first_receive) = bounded(1);
        let (second_send, second_receive) = bounded(1);

        let _ = thread::spawn({
            clone_all!(ast);
            move || {
                first_send.send(Interpreter::exec_ast(&ast)).ok();
            }
        });

        let _ = thread::spawn({
            clone_all!(ast);
            move || {
                second_send
                    .send(Jit::exec_ast(&ast, optimization_level.into()))
                    .ok();
            }
        });

        let result = select! {
            recv(first_receive) -> result => {
                debug!("Interpreter won: {:?}", result);
                result.unwrap()
            },
            recv(second_receive) -> result => {
                debug!("JIT won: {:?}", result);
                result.unwrap()
            }
        };

        debug!(
            "Hybrid exection finished in {} secs",
            time::precise_time_s() - current_time
        );
        result
    }
}
