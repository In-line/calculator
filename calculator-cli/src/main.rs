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

use ansi_term::Color;
use calculator_engine::execution::hybrid::{Hybrid, JitOptimizationLevel};

fn main() {
    let red = Color::Red.bold();
    let green = Color::Green;

    match Hybrid::exec(
        &std::env::args()
            .skip(1)
            .fold(String::new(), |mut acc, arg| {
                acc.push(' ');
                acc.push_str(&arg);
                acc
            }),
        JitOptimizationLevel::None,
    ) {
        Ok(result) => println!(
            "{prefix}{text}{suffix}",
            prefix = green.prefix(),
            text = result,
            suffix = green.suffix()
        ),
        Err(result) => println!(
            "{prefix}{text}{suffix}",
            prefix = red.prefix(),
            text = result,
            suffix = red.suffix()
        ),
    };
}
