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

use calculator_engine::{
    execution::hybrid::{Hybrid, JitOptimizationLevel},
    parser::Operator,
};

use gtk::{
    ApplicationWindow, BuilderExtManual, Button, ButtonExt, Inhibit, TextBufferExt, TextView,
    TextViewExt, WidgetExt,
};
use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;

#[derive(Msg)]
enum Msg {
    AddNumber(usize),
    AddOperator(Operator),
    AddText(&'static str),
    DoCalculation,
    Quit,
}

struct Widgets {
    window: ApplicationWindow,
    text_view_top: TextView,
    text_view_bottom: TextView,
}

struct Window {
    widgets: Widgets,
}

impl Update for Window {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: Self::ModelParam) -> Self::Model {}

    fn update(&mut self, event: Self::Msg) {
        let top_buffer = self.widgets.text_view_top.get_buffer().unwrap();

        match event {
            Msg::AddNumber(n) => top_buffer.insert_at_cursor(&n.to_string()),
            Msg::AddOperator(operator) => top_buffer.insert_at_cursor(match operator {
                Operator::Plus => " + ",
                Operator::Minus => " - ",
                Operator::Divide => " / ",
                Operator::Multiply => " * ",
            }),
            Msg::DoCalculation => {}
            Msg::AddText(text) => top_buffer.insert_at_cursor(text),
            Msg::Quit => gtk::main_quit(),
        }

        self.widgets
            .text_view_bottom
            .get_buffer()
            .unwrap()
            .set_text(&match Hybrid::exec(
                &top_buffer
                    .get_text(
                        &top_buffer.get_start_iter(),
                        &top_buffer.get_end_iter(),
                        false,
                    )
                    .unwrap()
                    .to_string(),
                JitOptimizationLevel::None,
            ) {
                Ok(result) => result.to_string(),
                Err(_) => "".to_owned(),
            });
    }
}

impl Widget for Window {
    type Root = ApplicationWindow;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, _: Self::Model) -> Self {
        let glade_src = include_str!("../glade/main.glade");
        let builder = gtk::Builder::new_from_string(glade_src);

        let window: ApplicationWindow = builder.get_object("main_window").unwrap();

        let text_view_top: TextView = builder.get_object("text_view_top").unwrap();

        connect!(
            relm,
            text_view_top.get_buffer().unwrap(),
            connect_changed(_),
            Msg::DoCalculation
        );

        for name in ["add", "sub", "div", "mul"].into_iter() {
            let operator = match *name {
                "add" => Operator::Plus,
                "sub" => Operator::Minus,
                "div" => Operator::Divide,
                "mul" => Operator::Multiply,
                _ => unreachable!(),
            };
            connect!(
                relm,
                builder
                    .get_object::<Button>(&format!("button_{}", name))
                    .unwrap(),
                connect_clicked(_),
                Msg::AddOperator(operator)
            );
        }

        for i in 0..10 {
            connect!(
                relm,
                builder
                    .get_object::<Button>(&format!("button_{}", i))
                    .unwrap(),
                connect_clicked(_),
                Msg::AddNumber(i)
            );
        }

        connect!(
            relm,
            builder.get_object::<Button>("button_left_parent").unwrap(),
            connect_clicked(_),
            Msg::AddText("(")
        );

        connect!(
            relm,
            builder.get_object::<Button>("button_right_parent").unwrap(),
            connect_clicked(_),
            Msg::AddText(")")
        );

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        window.show_all();

        Window {
            widgets: Widgets {
                window,
                text_view_top,
                text_view_bottom: builder.get_object("text_view_bottom").unwrap(),
            },
        }
    }
}

fn main() {
}
