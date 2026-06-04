// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
use my_parser::Parser;

#[test]
fn parse_hello_world() {
    let source = r#"
    fn main() {
        println("Hello");
    }
    "#;

    let mut parser = Parser::new(source);
    assert!(parser.parse_program().is_ok());
}
