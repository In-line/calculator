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
use linefeed::{Interface, ReadResult, Signal};
use pretty_env_logger::init;
use std::sync::Arc;
use std::time::Duration;

fn get_prompt(draw_red: bool) -> String {
    let red_style = Color::Red.bold();
    let green_style = Color::Green;

    let prompt_text = "> ";

    format!(
        "\x01{red_suffix}{green_suffix}{prompt_prefix}\x02{prompt_text}\x01{text_prefix}\x02",
        prompt_text = prompt_text,
        prompt_prefix = red_style.prefix(),
        text_prefix = if draw_red {
            red_style.prefix()
        } else {
            green_style.prefix()
        },
        red_suffix = red_style.suffix(),
        green_suffix = green_style.suffix(),
    )
}

fn main() -> std::io::Result<()> {
    init();

    let interface = Arc::new(Interface::new("calculator-repl")?);

    interface.set_prompt(&get_prompt(false))?;

    interface.set_history_size(10000);

    let mut last_buffer = String::new();

    loop {
        match interface.read_line_step(Some(Duration::from_millis(30)))? {
            Some(line) => match line {
                ReadResult::Input(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    match Hybrid::exec(&line, JitOptimizationLevel::None) {
                        Ok(result) => println!("{}", result),
                        Err(e) => println!("{}", e),
                    };

                    interface.add_history(line);
                }
                ReadResult::Signal(signal) => match signal {
                    Signal::Break
                    | Signal::Suspend
                    | Signal::Interrupt
                    | Signal::Resize
                    | Signal::Quit => return Ok(()),
                    Signal::Continue => {}
                },
                ReadResult::Eof => return Ok(()),
            },

            None => {
                let buffer = interface.buffer();

                if buffer != last_buffer {
                    interface.set_prompt(&get_prompt(
                        Hybrid::exec(&buffer, JitOptimizationLevel::None).is_err(),
                    ))?;

                    last_buffer = buffer;
                }
            }
        }
    }
}
