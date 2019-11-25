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

use super::errors::Error;
use super::parser::{parse, Operator, Token};
use log::*;
use snafu::{OptionExt, Snafu};
use std::iter::Peekable;
use std::vec::IntoIter as VecIter;
use std::{fmt, fmt::Formatter};

#[derive(Clone, Debug)]
pub enum Ast {
    Number(f64),
    BinaryOperator {
        operator: Operator,
        left: Box<Ast>,
        right: Box<Ast>,
    },
    UnaryOperator {
        operator: Operator,
        child: Box<Ast>,
    },
    Parenthesis {
        child: Box<Ast>,
    },
}

impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        fn display(this: &Ast) -> String {
            match this {
                Ast::Number(n) => format!("{}", n),
                Ast::BinaryOperator {
                    left,
                    right,
                    operator,
                } => format!("{} {} {}", left, operator, right),
                Ast::UnaryOperator { child, operator } => format!("{}{}", operator, child),
                Ast::Parenthesis { child } => format!("( {} )", child),
            }
        }

        write!(f, "{}", display(self))
    }
}

#[derive(Snafu, Debug, Clone)]
pub enum AstError {
    #[snafu(display("Expected next token, but got nothing"))]
    ExpectedToken,

    #[snafu(display("Expected operator, but got token: {:?}", token))]
    ExpectedOperator { token: Token },

    #[snafu(display("Unsupported unary operator: {:?}", operator))]
    UnsupportedUnaryOperator { operator: Operator },

    #[snafu(display("Unmatched closing parenthesis"))]
    UnmatchedClosingParenthesis,

    #[snafu(display(
        "Unmatched opening parenthesis, missing {} closing parenthesis",
        counter
    ))]
    UnmatchedOpeningParenthesis { counter: usize },
}

pub struct AstBuilder {
    token_iter: Peekable<VecIter<Token>>,
}

impl AstBuilder {
    pub fn build_ast(s: &str) -> Result<Ast, Error> {
        debug!("Starting to parse string {}", s);

        let (_, tokens) = parse(s).map_err(|err| match err {
            nom::Err::Failure(e) => e,
            nom::Err::Error(e) => e,
            _ => unimplemented!(),
        })?;

        debug!("Got tokens {:?}", tokens);

        AstBuilder {
            token_iter: tokens.into_iter().peekable(),
        }
        .expr(0)
    }

    pub fn build_ast_from_tokens(tokens: Vec<Token>) -> Result<Ast, Error> {
        AstBuilder {
            token_iter: tokens.into_iter().peekable(),
        }
        .expr(0)
    }

    fn nud(&mut self, t: Token) -> Result<Ast, Error> {
        match t {
            Token::Number(n) => Ok(Ast::Number(n)),
            Token::Operator(operator) => match operator {
                Operator::Plus | Operator::Minus => {
                    let right = self.expr(0)?;

                    Ok(Ast::UnaryOperator {
                        child: Box::new(right),
                        operator,
                    })
                }
                operator => Err(AstError::UnsupportedUnaryOperator { operator }.into()),
            },
            Token::OpenParenthesis => {
                let mut parenthesis = Vec::new();
                let mut counter: usize = 1;

                while let Some(token) = self.token_iter.next() {
                    match &token {
                        Token::OpenParenthesis => counter += 1,
                        Token::CloseParenthesis => {
                            match counter {
                                1 => {
                                    return AstBuilder::build_ast_from_tokens(parenthesis)
                                        .map(|t| Ast::Parenthesis { child: Box::new(t) });
                                }
                                0 => return Err(AstError::UnmatchedClosingParenthesis.into()),
                                _ => {}
                            };
                            counter -= 1;
                        }
                        _ => {}
                    };
                    parenthesis.push(token);
                }

                if counter != 0 {
                    Err(AstError::UnmatchedOpeningParenthesis { counter }.into())
                } else {
                    unreachable!()
                }
            }
            Token::CloseParenthesis => Err(AstError::UnmatchedClosingParenthesis.into()),
        }
    }

    fn led(&mut self, bp: usize, left: Ast, op: Token) -> Result<Ast, Error> {
        match op {
            Token::Operator(operator) => {
                let right = self.expr(bp)?;

                Ok(Ast::BinaryOperator {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator,
                })
            }
            token => Err(AstError::ExpectedOperator { token }.into()),
        }
    }

    fn expr(&mut self, rbp: usize) -> Result<Ast, Error> {
        let first_token = self.token_iter.next().context(ExpectedToken)?;
        let mut left = self.nud(first_token)?;

        while let Some(peeked) = self.token_iter.peek() {
            if rbp >= peeked.precedence() {
                break;
            }

            let op = self.token_iter.next().unwrap();
            left = self.led(op.precedence(), left, op)?;
        }

        Ok(left)
    }
}

pub fn build_ast(s: impl AsRef<str>) -> Result<Ast, Error> {
    AstBuilder::build_ast(s.as_ref())
}

#[cfg(test)]
mod tests {
    use super::AstBuilder;

    fn test_expr(s: &str) {
        assert_eq!(s, format!("{}", AstBuilder::build_ast(s).unwrap()));
    }

    fn check_error_type(s: &str, error_str: &str) {
        assert_eq!(
            format!("{:?}", AstBuilder::build_ast(s).unwrap_err()),
            error_str
        )
    }

    #[test]
    fn test_simple_expression() {
        test_expr("1 + 2 / 2 + 4");
        test_expr("1 / 2 / 2 - 4");
        test_expr("1 + 2 / 2 * 4");
    }

    #[test]
    fn test_expression_parenthesis() {
        test_expr("( 2 + 2 ) * 2");
        test_expr("2 * ( 2 + 2 )");
        test_expr("1 * ( 2 * ( 3 * ( 4 * 5 ) ) )");
    }

    #[test]
    fn check_failing() {
        check_error_type("b", "ParseError(Nom((\"b\", Many1)))");
    }
}
