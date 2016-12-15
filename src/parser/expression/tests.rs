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
        parser.unwrap()
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
        parser.unwrap()
    }}
}

macro_rules! has_command {
    ($commands:expr, $index:expr, $expected:expr) => (
        println!("Comparing command '{}' to expected command'{}'", $commands[$index], $expected)
        match $commands[$index] == format!($expected) {
            true => {
                println!("command[{}] '{}' was the expected input {}",
                    $index, $commands[$index], $expected);
            },
            false => {
                panic!("command[{}] '{}' was not the expected input {}",
                    $index, $commands[$index], $expected);
            }
        };
    );
}

macro_rules! is_commands {
    ($commands:expr, $($expected:expr),*) => (
        let mut index = 0;
        $(
            if index >= $commands.len() {
                panic!("Not enough commands were generated, only found {}.", index);
            }
            has_command!($commands, index, $expected);
            index += 1;
        )*
        if index != $commands.len() {
            println!("The parser generated {} more commands than were expected! Here are the extras:", index - $commands.len());
            for i in index..$commands.len() {
                println!("{:?}", $commands[i]);
            }
            panic!();
        }
    );
}

/// **************************************
/// ****** Expression Parsing Tests ******
/// **************************************

#[test]
#[should_panic]
// test if the expression parser works with empty expression (it should panic)
fn e_parser_empty() {
    eparser_helper!();
}

#[test]
#[should_panic]
// test if just an operand fails parsing
fn e_parser_operand() {
    eparser_helper!(Token::new_with(0, 0, "+".to_string(), TokenType::Plus));
}

#[test]
// Tests if the expression parser works with a single expression.
fn e_parser_single() {
    eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number));
}

#[test]
#[should_panic]
// Tests if the expression parser fails when there is an incomplete expression
fn e_parser_two_incomplete() {
    eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number),
        Token::new_with(0, 0, "+".to_string(), TokenType::Plus));
}

#[test]
// Tests if the expression parser can handle two values and an operator
fn e_parser_two() {
    eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number),
        Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
        Token::new_with(0, 0, "7".to_string(), TokenType::Number));
}

#[test]
// Test if the expression parser can handle an identifier
fn e_parser_identifier() {
    eparser_helper!(Token::new_with(0, 0, "5".to_string(), TokenType::Number),
        Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
        Token::new_with(0, 0, "7".to_string(), TokenType::Number),
        Token::new_with(0, 0, "*".to_string(), TokenType::Star),
        Token::new_with(0, 0, "x".to_string(), TokenType::Identifier));
}

#[test]
#[should_panic]
// Test if the expression parser can handle an identifier that is not in the symbol table
fn e_parser_identifier_fail() {
    let table = SymbolTable::empty();

    eparser_helper!(T table, Token::new_with(0, 0, "5".to_string(), TokenType::Number),
        Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
        Token::new_with(0, 0, "7".to_string(), TokenType::Number),
        Token::new_with(0, 0, "*".to_string(), TokenType::Star),
        Token::new_with(0, 0, "x".to_string(), TokenType::Identifier));
}

#[test]
// Tests "true"
fn e_parser_bool_one() {
    eparser_helper!(TS "true", TokenType::Keyword(KeywordType::True));
}

#[test]
// Tests "true AND false"
fn e_parser_bool_and() {
    eparser_helper!(TS "true", TokenType::Keyword(KeywordType::True),
        "AND", TokenType::Keyword(KeywordType::And),
        "false", TokenType::Keyword(KeywordType::False)
    );
}

#[test]
// Tests "5 < 4"
fn e_parser_int_comp() {
    eparser_helper!(TS "5", TokenType::Number,
        "<", TokenType::LessThan,
        "4", TokenType::Number
    );
}

#[test]
// Tests "5 < 4 == true" (it is false)
fn e_parser_combined() {
    eparser_helper!(TS
        "5", TokenType::Number,
        "<", TokenType::LessThan,
        "4", TokenType::Number,
        "==", TokenType::EqualTo,
        "true", TokenType::Keyword(KeywordType::True)
    );
}

#[test]
// Tests "5 < 4 == 10 >= 9" (it is false)
fn e_parser_int_comp_double() {
    eparser_helper!(TS
        "5", TokenType::Number,
        "<", TokenType::LessThan,
        "4", TokenType::Number,
        "==", TokenType::EqualTo,
        "10", TokenType::Number,
        ">=", TokenType::GreaterThanOrEqual,
        "9", TokenType::Number
    );
}

#[test]
// Tests "5 <= 4 + 1 == 10 >= 9" (it is true)
fn e_parser_int_comp_double_arith() {
    eparser_helper!(TS
        "5", TokenType::Number,
        "<", TokenType::LessThanOrEqual,
        "4", TokenType::Number,
        "==", TokenType::EqualTo,
        "10", TokenType::Number,
        ">=", TokenType::GreaterThanOrEqual,
        "9", TokenType::Number
    );
}

#[test]
// Tests "5 <= 4 + 1 == 2 + 10 >= 9 + 10" (it is false)
fn e_parser_int_comp_double_arith2() {
    eparser_helper!(TS
        "5", TokenType::Number,
        "<", TokenType::LessThanOrEqual,
        "4", TokenType::Number,
        "==", TokenType::EqualTo,
        "10", TokenType::Number,
        ">=", TokenType::GreaterThanOrEqual,
        "9", TokenType::Number
    );
}

/// *****************************************************
/// ****** Expression Parser Code Generation Tests ******
/// *****************************************************

#[test]
// Checks if a single identifier will generate the correct code
//
// let parser = ... will push x to stack
fn code_single_expression() {
    let parser = eparser_helper!(Token::new_with(0, 0, "x".to_string(), TokenType::Identifier));
    let c = parser.commands;

    // Doesn't need to do any stack work if x is already on the stack
    assert!(c.len() == 0);
}

#[test]
// Checks if adding two identifiers will generate the correct code
fn code_add_two() {
    let parser = eparser_helper!(Token::new_with(0, 0, "x".to_string(), TokenType::Identifier),
                                Token::new_with(0, 0, "+".to_string(), TokenType::Plus),
                                Token::new_with(0, 0, "y".to_string(), TokenType::Identifier));

    // Load value of x to temporary $0 => movw +0@R0 +0@R1
    // Add value of y to temporary $0 => addw +4@R0 +0@R1
    is_commands!(parser.commands, "movw +0@R0 +0@R1",
        "addw +4@R0 +0@R1");
}

#[test]
// Check if we can produce the correct code for product of two
fn code_product_two() {
    let parser = eparser_helper!(Token::new_with(0, 0, "x".to_string(), TokenType::Identifier),
                                Token::new_with(0, 0, "*".to_string(), TokenType::Star),
                                Token::new_with(0, 0, "y".to_string(), TokenType::Identifier));

    // Load value of x to temporary $0 => movw +0@R0 +0@R1
    // Add value of y to temporary $0 => mulw +4@R1 +0@R1
    is_commands!(parser.commands,
        "movw +0@R0 +0@R1",
        "mulw +4@R0 +0@R1");
}

#[test]
// Check if we can produce the correct code for x mod y
fn code_mod_two() {
    let parser = eparser_helper!(TS "x", TokenType::Identifier,
        "mod", TokenType::Keyword(KeywordType::Mod),
        "y", TokenType::Identifier);

    // Move x to temp variable
    // Divide temp by y
    // Multiply temp by y
    // Move x to second temp variable
    // Subtract first temp from second temp
    // Move second temp to R1
    is_commands!(parser.commands,
        "movw +0@R0 +0@R1",
        "divw +4@R0 +0@R1",
        "mulw +4@R0 +0@R1",
        "movw +0@R0 +4@R1",
        "subw +0@R1 +4@R1",
        "movw +4@R1 +0@R1"
    );
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
    );

    // Move y to temp 1
    // Mult temp 1 by z
    // Add x to temp 1
    is_commands!(parser.commands,
        // Move y to temp1
        "movw +4@R0 +0@R1",

        // Multiply temp1 by z
        "mulw +8@R0 +0@R1",

        // WOULD BE THIS but we can not assume a * b = b * a so we must move to a second temp
        // variable instead
        // Add x to temp1
        //"addw +0@R0 +0@R1"

        // Move x to temp2
        "movw +0@R0 +4@R1",

        // Add temp1 to temp2
        "addw +0@R1 +4@R1",

        // Move temp2 to R1
        "movw +4@R1 +0@R1"
    );
}

#[test]
// Check if we can produce correct code for a long operation
// 4 + x * y - 30 div z + 1
// (from testG.txt)
fn code_long_expression() {
    let parser = eparser_helper!(TS
        "4", TokenType::Number,
        "+", TokenType::Plus,
        "x", TokenType::Identifier,
        "*", TokenType::Star,
        "y", TokenType::Identifier,
        "-", TokenType::Minus,
        "30", TokenType::Number,
        "div", TokenType::Keyword(KeywordType::Div),
        "z", TokenType::Identifier,
        "+", TokenType::Plus,
        "1", TokenType::Number
    );

    // move x to temp 1
    // mult temp 1 by y
    // add temp 1 by 4
    // move 30 to temp 2
    // div temp 2 by z
    // sub temp 1 by temp 2
    // add 1 to temp 1
    is_commands!(parser.commands,
        // move x to temp1
        "movw +0@R0 +0@R1",

        // mult temp1 by y
        "mulw +4@R0 +0@R1",

        // move 30 to t2
        "movw ^30 +4@R1",

        // div temp2 by z
        "divw +8@R0 +4@R1",

        // move 1 to t3
        "movw ^1 +8@R1",

        // add t3 to t2
        "addw +8@R1 +4@R1",

        // sub t2 from t1
        "subw +4@R1 +0@R1",

        // move 4 to t4
        "movw ^4 +12@R1",

        // add t1 to t4 (because of the order of operations)
        "addw +0@R1 +12@R1",

        // move t4 to +0@R1
        "movw +12@R1 +0@R1"
    );
}
