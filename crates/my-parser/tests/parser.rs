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
