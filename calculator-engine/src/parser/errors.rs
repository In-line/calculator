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

use nom::error::{ErrorKind as NomErrorKind, ErrorKind};
use std::fmt;

type NomError<I> = (I, NomErrorKind);

use super::ParseUserError;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError<I = String> {
    Nom(NomError<I>),
    User(ParseUserError),
}

impl<I: fmt::Debug + fmt::Display> std::error::Error for ParseError<I> {}

impl<I: fmt::Display> fmt::Display for ParseError<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ParseError::User(u) => write!(f, "{}", u),
            ParseError::Nom((i, _)) => write!(f, "Parser error near token: {}", i),
        }
    }
}

impl<I> From<NomError<I>> for ParseError<I> {
    fn from(err: NomError<I>) -> Self {
        ParseError::Nom(err)
    }
}

impl<I> From<ParseUserError> for ParseError<I> {
    fn from(err: ParseUserError) -> Self {
        ParseError::User(err)
    }
}

impl<'a> From<ParseError<&'a str>> for ParseError<String> {
    fn from(err: ParseError<&'a str>) -> ParseError<String> {
        match err {
            ParseError::User(err) => ParseError::User(err),
            ParseError::Nom((data, error_kind)) => ParseError::Nom((data.to_owned(), error_kind)),
        }
    }
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        ParseError::Nom((input, kind))
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

pub fn parse_error_to_owned<'a, I: ToOwned + ?Sized + 'a>(
    err: nom::Err<ParseError<&I>>,
) -> nom::Err<ParseError<I::Owned>> {
    fn error_to_owned<'a, I: ToOwned + ?Sized + 'a>(err: ParseError<&I>) -> ParseError<I::Owned> {
        match err {
            ParseError::User(err) => ParseError::User(err),
            ParseError::Nom((i, err)) => ParseError::Nom((i.to_owned(), err)),
        }
    }

    match err {
        nom::Err::Error(err) => nom::Err::Error(error_to_owned(err)),
        nom::Err::Failure(err) => nom::Err::Failure(error_to_owned(err)),
        nom::Err::Incomplete(n) => nom::Err::Incomplete(n),
    }
}
