/// parser/expression/tests.rs
///
/// This file contains unit tests for the expression parser for correct parsing as well
/// as correct assembly code generation from that parsing.

use super::*;

macro_rules! eparser_helper {
    ($( $token:expr ),*) => {{
        let mut tokens = Vec::<Token>::new();
        $(
            tokens.push($token);
        )*
        let mut table = SymbolTable::empty();
        for t in tokens.iter() {
            if t.is_type(TokenType::Identifier) {
                table.add(t.lexeme(), SymbolType::Variable(SymbolValueType::Int));
            }
        }

        let parser = ExpressionParser::new(table, tokens);
        parser
    }};
}

#[test]
#[should_panic]
// test if the expression parser works with empty expression (it should panic)
fn e_parser_empty() {
    let parser = eparser_helper!();
    assert!(parser.is_none())
}

#[test]
#[should_panic]
// test if just an operand fails parsing
fn e_parser_operand() {
    let parser = eparser_helper!(Token::new_with(0, 0, "+".to_string(), TokenType::Plus));
    assert!(parser.is_some());
}

#[test]
// Tests if the expression parser works with a single expression.
fn e_parser_single() {
    let parser = eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number));

    assert!(parser.is_some());
}

#[test]
#[should_panic]
// Tests if the expression parser fails when there is an incomplete expression
fn e_parser_two_incomplete() {
    let parser = eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number),
                                Token::new_with(0, 0, "+".to_string(), TokenType::Plus));

    assert!(parser.is_some());
}

#[test]
// Tests if the expression parser can handle two values and an operator
fn e_parser_two() {
    let parser = eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number),
                                Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
                                Token::new_with(0, 0, "7".to_string(), TokenType::Number));

    assert!(parser.is_some());
}

#[test]
// Test if the expression parser can handle an identifier
fn e_parser_identifier() {
    let parser = eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number),
                                Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
                                Token::new_with(0, 0, "7".to_string(), TokenType::Number),
                                Token::new_with(0, 0, "*".to_string(), TokenType::Star),
                                Token::new_with(0, 0, "x".to_string(), TokenType::Identifier));

    assert!(parser.is_some());
}
