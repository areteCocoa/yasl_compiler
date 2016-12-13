/// parser/expression/tests.rs
///
/// This file contains unit tests for the expression parser for correct parsing as well
/// as correct assembly code generation from that parsing.

use super::*;

/// Helper macro for generating a parser based on the set of tokens, or a given
/// symbol table and a set of tokens.
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
    (TS $( $s:expr, $t:expr ), *) => {{
        eparser_helper!(
            $(
                Token::new_with(0, 0, format!("{}", $s), $t)
            ),*
        )
    }};
    (T $table:ident, $( $token:expr ), *) => {{
        let mut tokens = Vec::<Token>::new();
        $(
            tokens.push($token);
        )*

        let parser = ExpressionParser::new($table, tokens);
        parser
    }}
}

macro_rules! has_command {
    ($commands:expr, $index:expr, $expected:expr) => (
        match $commands[$index] == format!($expected) {
            true => {},
            false => {
                panic!("command[{}] '{}' was not the expected input {}",
                    $index, $commands[$index], $expected);
            }
        };
    );
}

/// **************************
/// ****** Parser Tests ******
/// **************************

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

#[test]
#[should_panic]
// Test if the expression parser can handle an identifier that is not in the symbol table
fn e_parser_identifier_fail() {
    let mut table = SymbolTable::empty();

    let parser = eparser_helper!(T table, Token::new_with(0, 0, "5".to_string(), TokenType::Number),
                                Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
                                Token::new_with(0, 0, "7".to_string(), TokenType::Number),
                                Token::new_with(0, 0, "*".to_string(), TokenType::Star),
                                Token::new_with(0, 0, "x".to_string(), TokenType::Identifier));

    assert!(parser.is_some())
}

/// ******************************************
/// ****** Parser Code Generation Tests ******
/// ******************************************

#[test]
// Checks if a single identifier will generate the correct code
fn code_single_expression() {
    let parser = eparser_helper!(Token::new_with(0, 0, "x".to_string(), TokenType::Identifier)).unwrap();
    let c = parser.commands;

    unimplemented!();
}

#[test]
// Checks if adding two identifiers will generate the correct code
fn code_add_two() {
    let parser = eparser_helper!(Token::new_with(0, 0, "x".to_string(), TokenType::Identifier),
                                Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
                                Token::new_with(0, 0, "y".to_string(), TokenType::Identifier)).unwrap();

    // Code generation should push x and y to the stack and then add them to a temporary variable
    let commands = parser.commands;
    // TODO: Load x and y to the stack?
    // NOTE: Temporary symbol '$0' should be generated at +0@R1
    // Load value of x to temporary $0 => movw +0@R0 +0@R1
    // Add value of y to temporary $0 => addw +4@R1 +0@R1

    has_command!(commands, 0, "movw +0@R0 +0@R1");
    has_command!(commands, 1, "addw +4@R0 +0@R1");
}

#[test]
// Check if we can produce the correct code for product of two
fn code_product_two() {
    let parser = eparser_helper!(Token::new_with(0, 0, "x".to_string(), TokenType::Identifier),
                                Token::new_with(0, 0, "*".to_string(), TokenType::Star),
                                Token::new_with(0, 0, "y".to_string(), TokenType::Identifier)).unwrap();

    let commands = parser.commands;
    // TODO: Load x and y to the stack?
    // NOTE: Temporary symbol '$0' should be generated at +0@R1
    // Load value of x to temporary $0 => movw +0@R0 +0@R1
    // Add value of y to temporary $0 => mulw +4@R1 +0@R1
    assert!(commands[0] == format!("movw +0@R0 +0@R1") && commands[1] == format!("mulw +4@R0 +0@R1"));
}

#[test]
// Check if we can produce the correct code for x mod y
fn code_mod_two() {
    unimplemented!();
}

#[test]
// Check if we can produce the correct code with order of operations for x + y * z
fn code_add_product_three() {
    let parser = eparser_helper!(
        Token::new_with(0, 0, "x".to_string(), TokenType::Identifier),
        Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
        Token::new_with(0, 0, "y".to_string(), TokenType::Identifier),
        Token::new_with(0, 0, "*".to_string(), TokenType::Star),
        Token::new_with(0, 0, "z".to_string(), TokenType::Identifier)
    ).unwrap();

    //
    unimplemented!();
}

#[test]
// Check if we can produce correct code for a long operation
// 4 + x * y - 30 div z + 1
// (from testG.txt)
fn code_long_expression() {
    let parser = eparser_helper!(TS "4", TokenType::Number);

    unimplemented!();
}
