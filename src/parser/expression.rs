/*
 * Expression parser
 *
 */

use super::{Token, TokenType, KeywordType};
use super::{Symbol, SymbolTable, SymbolType};

use std::cmp::Ordering;
use std::fmt;

 static mut VERBOSE: bool = true;

 macro_rules! log {
     ($message:expr $(,$arg:expr)*) => {
         unsafe {
             if VERBOSE == true {
                 println!($message, $($arg,)*);
             }
         }
     };
 }

#[derive(PartialEq, Clone)]
enum Expression {
    // +, -, etc
    // TokenType is wrapped to store what kind of operator this is
    Operator(TokenType),

    // 2, 5, 7.5, etc
    // The string is stored to have the value of the operand or its name
    Operand(Token),

    // A combined expression using three other expressions, in
    // operand - operator - operand format
    Combined(Box<Expression>, Box<Expression>, Box<Expression>),
}

impl Expression {
    // Creates a new expression from a token
    fn from_token(t: Token) -> Option<Expression> {
        match t.token_type() {
            TokenType::Number => Some(Expression::Operand(t)),

            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Keyword(KeywordType::Div)
            | TokenType::Keyword(KeywordType::Mod) | TokenType::GreaterThan | TokenType::LessThan
            | TokenType::GreaterThanOrEqual | TokenType::LessThanOrEqual | TokenType::EqualTo
            | TokenType::NotEqualTo => Some(Expression::Operator(t.token_type())),


            TokenType::Keyword(KeywordType::Print) => Some(Expression::Operator(t.token_type())),

            TokenType::Identifier => Some(Expression::Operand(t)),

            _ => {
                None
            },
        }
    }
}

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Expression) -> Option<Ordering> {
        use self::Expression::*;

        match self {
            // *, div, mod (4)
            &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
            | &Operator(TokenType::Keyword(KeywordType::Mod)) => {
                // * or div or mod
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))|
                    &Operator(TokenType::Keyword(KeywordType::Mod)) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Greater),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Greater),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Greater),

                    &Operand(_) => Some(Ordering::Less),

                    _ => None,
                }
            },

            // +, - (3)
            &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => {
                // + or -
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
                    | &Operator(TokenType::Keyword(KeywordType::Mod))
                    | &Operand(_) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Less),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Greater),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Greater),

                    _ => None,
                }
            },

            // >, <, >=, <= (2)
            &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
            | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual) => {
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
                    | &Operator(TokenType::Keyword(KeywordType::Mod))
                    | &Operand(_) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Less),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Less),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Greater),

                    _ => None,
                }
            },

            &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => {
                match other {
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))
                    | &Operator(TokenType::Keyword(KeywordType::Mod))
                    | &Operand(_) => Some(Ordering::Less),

                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) => Some(Ordering::Less),

                    &Operator(TokenType::GreaterThan) | &Operator(TokenType::LessThan)
                    | &Operator(TokenType::GreaterThanOrEqual) | &Operator(TokenType::LessThanOrEqual)
                        => Some(Ordering::Less),

                    &Operator(TokenType::EqualTo) | &Operator(TokenType::NotEqualTo) => Some(Ordering::Less),

                    _ => None,
                }
            }

            &Operand(_) => {
                // Any number
                match other {
                    &Operator(TokenType::Plus) | &Operator(TokenType::Minus) |
                    &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))|
                    &Operator(TokenType::Keyword(KeywordType::Mod)) => Some(Ordering::Greater),

                    &Operand(_) => None,

                    _ => None,
                }
            }

            _ => None,
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Expression::Operator(ref t) => {
                write!(f, "{}", t)
            },
            &Expression::Operand(ref v) => {
                write!(f, "{}", v)
            }
            &Expression::Combined(ref o1, ref o, ref o2) => {
                write!(f, "{} {} {}", *o1, *o, *o2)
            }
        }
    }
}

// ExpressionParser validates the syntax of an expression as well as reduces it and
// manages memory allocation for temporary variables used for arithmatic
pub struct ExpressionParser {

}

impl ExpressionParser {
    // Creates a new ExpressionParser given the tokens and parses through them
    pub fn new(table: SymbolTable, tokens: Vec<Token>) -> Option<ExpressionParser> {
        // Convert the tokens into expressions
        let expressions = match ExpressionParser::tokens_to_expressions(tokens) {
            Some(e) => e,
            None => return None,
        };

        // Convert infix notation to reverse polish notation
        let mut postfix_exp = match ExpressionParser::expressions_to_postfix(expressions) {
            Some(e) => e,
            None => return None,
        };

        // Reduce the stack of expressions until there is only one remaining
        let mut stack = Vec::<Expression>::new();
        while postfix_exp.len() > 0 {
            // Pop the first expression
            let e = postfix_exp.remove(0);

            match e.clone() {
                Expression::Operand(t) => {
                    // Check that the variable has been declared
                    if let Some(s) = table.get(&*t.lexeme()) {
                        match s.symbol_type {
                            SymbolType::Procedure => {
                                // We can't use procedures in expressions, fail
                                panic!("Attempted to use a procedure as a variable in an expression!");
                            }
                            _ => {}
                        }
                        // Push the operand to the stack
                        stack.push(e);
                    } else {
                        panic!("Attempted to use a variable that has not been declared!");
                    }
                },
                Expression::Operator(_) => {
                    // Pop the previous two expressions and combine them
                    let e1 = match stack.pop() {
                        Some(s) => s,
                        None => return None, // TODO: PRINT MESSAGE
                    };
                    let e2 = match stack.pop() {
                        Some(s) => s,
                        None => return None, // TODO: PRINT MESSAGE
                    };

                    // Get the temporary symbol from the symbol table
                    println!("<YASLC/ExpressionParser> WARNING: Did not generate a temporary symbol for the expression!");

                    // Create the output for the evaluation using the temp symbol

                    // Create the combination expression
                    let c = Expression::Combined(Box::new(e1), Box::new(e), Box::new(e2));

                    // Push the combination expression to the stack
                    stack.push(c);
                },
                Expression::Combined(_, _, _) => {
                    stack.push(e);
                }
            }
        }

        Some(ExpressionParser {

        })
    }

    // Converts the vector of tokens to a vector of expressions and returns None if there was an
    // invalid token
    fn tokens_to_expressions(mut tokens: Vec<Token>) -> Option<Vec<Expression>> {
        let mut expressions = Vec::<Expression>::new();
        // while there's still tokens, push them onto the stack
        while tokens.len() > 0 {
            // Get the front token
            let t = tokens.remove(0);

            log!("{}",t);

            // Attempt to convert it to an expression
            if let Some(e) = Expression::from_token(t) {
                expressions.push(e);
            } else {
                // Converting to an expression failed likely because it is an invalid token
                // Stop all conversion and return
                println!("<YASLC/ExpressionParser> Error: invalid token in expression.");
                return None;
            }
        }

        log!("<YASLC/ExpressionParser> Successfully converted tokens to expressions!");

        Some(expressions)
    }

    // Converts the vector of expressions to postfix from infix.
    fn expressions_to_postfix(expressions: Vec<Expression>) -> Option<Vec<Expression>> {
        // Initialize the stack and the operator stack
        let mut stack: Vec<Expression> = Vec::<Expression>::new();
        let mut op_stack: Vec<Expression> = Vec::<Expression>::new();

        for e in expressions {
            // We have expression e, match what is is
            match e {
                Expression::Operand(_) => {
                    // We have a number, push to the stack
                    stack.push(e);
                },
                Expression::Operator(_) => {
                    // We have an operator, check it's precedence vs the top of the stack
                    if op_stack.len() != 0 {
                        while op_stack[op_stack.len() - 1] <= e {
                            if let Some(o) = op_stack.pop() {
                                log!("<YASLC/ExpressionParser> Pushing operator to the operand stack.");
                                stack.push(o);
                            } else {
                                // We emptied the operator stack successfully, break the loop
                                break;
                            }
                        }
                    }

                    op_stack.push(e);
                },
                _ => {
                    println!("YASLC/ExpressionParser> Error: Found combined expression while converting to postfix!");
                    return None;
                }
            }
        }

        // Check if we have any operators left in the op_stack and just push the to the stack in order
        while let Some(o) = op_stack.pop() {
            stack.push(o);
        }

        log!("<YASLC/ExpressionParser> Successfully converted infix expressions to postfix.");

        Some(stack)
    }


}
