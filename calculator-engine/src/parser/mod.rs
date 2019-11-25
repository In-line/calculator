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

mod errors;
pub use errors::*;

use derive_more::From;
use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{char, one_of},
    combinator::map,
    multi::{fold_many1, many0},
    number::complete::double,
    sequence::tuple,
};
use snafu::Snafu;
use std::fmt;
use std::fmt::Formatter;

type IResult<'a, O, E = ParseError<&'a str>> = nom::IResult<&'a str, O, E>;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operator {
    Minus,
    Plus,
    Divide,
    Multiply,
}

impl Operator {
    pub fn precedence(self) -> usize {
        match self {
            Operator::Minus | Operator::Plus => 1,
            Operator::Divide | Operator::Multiply => 2,
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Operator::Minus => '-',
                Operator::Plus => '+',
                Operator::Divide => '/',
                Operator::Multiply => '*',
            }
        )
    }
}

#[derive(Clone, Debug)]
pub enum Token {
    Number(f64),
    Operator(Operator),
    OpenParenthesis,
    CloseParenthesis,
}
impl Token {
    pub fn precedence(&self) -> usize {
        match self {
            Token::Operator(op) => op.precedence(),
            _ => usize::max_value(),
        }
    }
}
#[derive(Snafu, Debug, From, Clone, PartialEq)]
pub enum ParseUserError {
    #[snafu(display("Invalid operator: {}", operator))]
    InvalidOperator { operator: char },
}
fn parse_operator(s: &str) -> IResult<Operator> {
    let (s, c) = take(1 as usize)(s)?;
    assert_eq!(c.len(), 1);
    Ok((
        s,
        match c.chars().next().unwrap() {
            '+' => Ok(Operator::Plus),
            '-' => Ok(Operator::Minus),
            '*' => Ok(Operator::Multiply),
            '/' => Ok(Operator::Divide),
            operator => Err(nom::Err::Error(
                ParseUserError::InvalidOperator { operator }.into(),
            )),
        }?,
    ))
}
fn parse_number(s: &str) -> IResult<f64> {
    double(s)
}
fn skip_whitespace(s: &str) -> IResult<()> {
    Ok((many0(one_of(" \t\x0c\n"))(s)?.0, ()))
}
pub fn parse(s: &str) -> IResult<Vec<Token>, ParseError> {
    fold_many1(
        map(
            tuple((
                skip_whitespace,
                alt((
                    map(parse_operator, Token::Operator),
                    map(parse_number, Token::Number),
                    map(char('('), |_| Token::OpenParenthesis),
                    map(char(')'), |_| Token::CloseParenthesis),
                )),
            )),
            |((), token)| token,
        ),
        Vec::new(),
        |mut acc, token| {
            acc.push(token);
            acc
        },
    )(s)
    .map_err(|e: nom::Err<ParseError<&str>>| {
        let e: nom::Err<ParseError> = parse_error_to_owned(e);
        e
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_skip_whitespace() {
        assert_eq!("bla b ", skip_whitespace("   bla b ").unwrap().0);
        assert_eq!("bla", skip_whitespace("bla").unwrap().0);
    }
    #[test]
    fn test_operator() {
        assert_eq!(Operator::Plus, parse_operator("+12").unwrap().1);
        assert_eq!(Operator::Minus, parse_operator("-").unwrap().1);
        assert_eq!(Operator::Multiply, parse_operator("*").unwrap().1);
        assert_eq!(Operator::Divide, parse_operator("/").unwrap().1);
        assert!(parse_operator("b").is_err());
    }
}
