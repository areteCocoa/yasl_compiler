/// parser/expression.rs
///
/// The expression module contains the expression parser and all the implementation
/// for the expression parser.

#[cfg(test)]
mod tests;

pub use super::{Token, TokenType, KeywordType};
pub use super::{Symbol, SymbolTable, SymbolType, SymbolValueType};

use std::cmp::Ordering;
use std::fmt;

/// Set to true if you want the expression parser to print its process.
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

/// Expression represents a single piece of expressions.
#[derive(PartialEq, Clone)]
enum Expression {
    /// Operator are all expressions that modify Operands or Combined expressions.
    ///
    /// Example: +, -, etc
    ///
    /// TokenType is wrapped to store what kind of operator this is.
    Operator(TokenType),

    /// Operand is any expression that can be operated on by an operator.
    ///
    // Example: 2, 5, 7.5, etc
    ///
    // The string is stored to have the value of the operand or its name.
    Operand(Token),

    /// A combined expression using three other expressions, in
    /// operand - operator - operand format.
    Combined(Symbol),
}

impl Expression {
    /// Creates a new expression from a token and returns Some(e) where e is a valid expression,
    /// or returns None if the expression is not valid given the token.
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

// Define ordering for expressions because that is used in reducing expressions from postfix.
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
            &Expression::Combined(ref s) => {
                write!(f, "{:?}", s)
            }
        }
    }
}

/// ExpressionParser validates the syntax of an expression as well as reduces it and
/// manages memory allocation for temporary variables used for arithmatic.
pub struct ExpressionParser {
    /// The list of commands to be pushed onto the program given this expression.
    commands: Vec<String>,
}

impl ExpressionParser {
    /// Creates a new ExpressionParser given the tokens and parses through them. It returns
    /// Some(e) where e is a valid expression parser if there is no error and None otherwise.
    pub fn new(table: SymbolTable, tokens: Vec<Token>) -> Option<ExpressionParser> {
        // Convert the tokens into expressions
        let expressions = match ExpressionParser::tokens_to_expressions(tokens) {
            Some(e) => e,
            None => return None,
        };

        // Convert infix notation to reverse polish notation
        let postfix_exp = match ExpressionParser::expressions_to_postfix(expressions) {
            Some(e) => e,
            None => return None,
        };

        let (f_symbol_opt, mut commands) = ExpressionParser::reduce_expression_stack(postfix_exp, table);

        let f_symbol = match f_symbol_opt {
            Some(s) => s,
            None => {
                println!("<YASLC/ExpressionParser> Attempted to get final symbol for expression but none was found!");
                return None;
            }
        };

        // Now that we have one single expression, move it to the SP
        if f_symbol.offset() != 0 {
            let sp_mov = format!("movw +{}@R{} +0@R{}", f_symbol.offset(), f_symbol.register(), f_symbol.register());
            commands.push(sp_mov);
        }

        Some(ExpressionParser {
            commands: commands,
        })
    }

    /// Returns a list of commands from this expression parser.
    pub fn commands(&self) -> Vec<String> {
        self.commands.clone()
    }

    /// Reduces the stack of postfix expressions until there is only one remaining.
    fn reduce_expression_stack(mut expressions: Vec<Expression>, mut table: SymbolTable) -> (Option<Symbol>, Vec<String>) {
        // Move the register up by 1
        table.up_register();

        // Declare the stack and the list of commands
        let mut stack = Vec::<Expression>::new();
        let mut commands = Vec::<String>::new();

        // Reduce the list until there are no commands remaining
        while expressions.len() > 0 {
            // Pop the first expression
            let e = expressions.remove(0);

            match ExpressionParser::handle_expression(e, &mut table, &mut stack) {
                Ok(opt_com) => match opt_com {
                        Some(coms) => {
                            for com in coms {
                                commands.push(com);
                            }
                        },
                        None => {},
                    },
                Err(error) => {
                    panic!("<YASLC/ExpressionParser> Error handling expression: {}", error);
                }
            };

        }

        let f_symbol = match ExpressionParser::final_symbol(stack, table) {
            (Some(s), coms) => {
                for com in coms {
                    commands.push(com);
                }
                s
            },
            _ => {
                // TODO: Write something real
                panic!("UNEXPECTED RESULT");
            }
        };

        (Some(f_symbol), commands)
    }

    fn final_symbol(mut stack: Vec<Expression>, mut table: SymbolTable) -> (Option<Symbol>, Vec<String>) {
        match stack.remove(0) {
            Expression::Combined(s) => (Some(s), Vec::<String>::new()),
            Expression::Operand(t) => {
                // It is a single operand
                match t.is_type(TokenType::Identifier) {
                    // It is in the symbol table
                    true => {
                        let symbol = match table.get(&*t.lexeme()) {
                            Some(s) => s,
                            None => {
                                panic!("<YASLC/ExpressionParser> Attempted to use a symbol that
                                was not found in the symbol table! This is very unexpected...");
                            }
                        };
                        (Some(symbol.clone()), Vec::<String>::new())
                    },
                    // It is a number and we can use it literally
                    false => {
                        // TODO: Do we push the number to the stack as a temp?
                        let s = table.temp();
                        // TODO: Is the correct format for constants "^"?
                        let c = format!("movw ^{} +{}@R{}", t.lexeme(), s.offset(), s.register());
                        let mut commands = Vec::<String>::new();
                        commands.push(c);
                        (Some(s.clone()), commands)
                    }
                }
            }
            _ => {
                panic!("Found expression consisting of only an operator!");
            }
        }
    }

    fn last_two_expressions(stack: &mut Vec<Expression>) -> Result<(Expression, Expression), String> {
        let e2 = match stack.pop() {
            Some(s) => s,
            None => {
                return Err(format!("<YASLC/ExpressionParser> Error: attempted to reduce expression but there is a missing operand!"));
            }
        };
        let e1 = match stack.pop() {
            Some(s) => s,
            None => {
                return Err(format!("<YASLC/ExpressionParser> Error: attempted to reduce expression but there are two missing operands!"));
            }
        };
        Ok((e1, e2))
    }

    // fn perform_operation(t_type: TokenType, s1: &Symbol, v: String) -> Result<Vec<String>, String> {
    //     Err("".to_string())
    // }

    fn reduce_expression(t_type: TokenType, mut stack: &mut Vec<Expression>, mut table: &mut SymbolTable) -> Result<Vec<String>, String> {
        let mut commands = Vec::<String>::new();

        // Pop the previous two expressions
        let (e1, e2) = match ExpressionParser::last_two_expressions(&mut stack) {
            Ok((r1, r2)) => (r1, r2),
            Err(e) => panic!("<YASLC/ExpresionParser> {}", e),
        };

        // Match the first expression because if it is a temp variable we can operate on that
        // and not have to create another temp variable
        let s1 = match e1 {
            Expression::Operand(t) => {
                if t.is_type(TokenType::Identifier) {
                    match table.get(&*t.lexeme()) {
                        Some(x) => x.clone(),
                        None => panic!("Attempted to use variable '{}' that has not been declared!", t.lexeme()),
                    }
                } else {
                    // Generate the "value code for a constant number"
                    table.temp()
                }
            },
            Expression::Combined(s) => s,
            _ => panic!("Found an operator where we were expecting an operand!"),
        };

        let s2 = match e2 {
            Expression::Operand(t) => {
                if t.is_type(TokenType::Identifier) {
                    match table.get(&*t.lexeme()) {
                        Some(x) => x.clone(),
                        None => panic!("Attempted to use variable '{}' that has not been declared!", t.lexeme()),
                    }
                } else {
                    // Generate the "value code for a constant number"
                    table.temp()
                }
            },
            Expression::Combined(s) => s,
            _ => panic!("Found an operator where we were expecting an operand!"),
        };

        // Find the destination symbol
        let dest = if s1.is_temp() {
            // We can operate on s1
            println!("We can operate on {:?} for expression in place of a temp because it is already a temp!", s1);
            s1.clone()
        } else {
            // We have to operate on a temp
            //
            // Move the value from the first symbol to temp
            let temp = table.temp();
            let mov = format!("movw +{}@R{} +{}@R{}", s1.offset(), s1.register(),
                temp.offset(), temp.register());
            commands.push(mov);
            temp
        };

        // Determine the operator given the token type
        let op = match t_type {
            TokenType::Plus => "addw",
            TokenType::Minus => "subw",
            TokenType::Star => "mulw",
            TokenType::Keyword(KeywordType::Div) => "divw",
            TokenType::Keyword(KeywordType::Mod) => {
                // Special case, will return value for the function

                // Generate temp 1 and 2
                let temp1 = dest;
                let temp2 = table.temp();

                // Divide temp1 by s2
                commands.push(format!("divw +{}@R{} +{}@R{}",
                    s2.offset(), s2.register(), temp1.offset(), temp1.register()));

                // Multiply temp1 by s2
                commands.push(format!("mulw +{}@R{} +{}@R{}",
                    s2.offset(), s2.register(), temp1.offset(), temp1.register()));

                // Move s1 to temp2
                commands.push(format!("movw +{}@R{} +{}@R{}",
                    s1.offset(), s1.register(), temp2.offset(), temp2.register()));

                // Subtract temp1 from temp2
                commands.push(format!("subw +{}@R{} +{}@R{}",
                    temp1.offset(), temp1.register(), temp2.offset(), temp2.register()));

                // Move temp2 to s1 (dest)
                commands.push(format!("movw +{}@R{} +{}@R{}",
                    temp2.offset(), temp2.register(), temp1.offset(), temp1.register()));

                log!("<YASLC/ExpressionParser> Successfully generated 'mod' expression code.");

                // Generate the combined expression
                let c = Expression::Combined(temp1);
                stack.push(c);

                return Ok(commands);
            }
            n => panic!("Unrecognized operator '{}' in expression!", n),
        };

        // Perform the operation
        let full_op = format!("{} +{}@R{} +{}@R{}", op, s2.offset(), s2.register(),
            dest.offset(), dest.register());

        for c in commands.iter() {
            log!("YASLC/ExpressionParser> Generated code for reduction: '{}'", c);
        }
        log!("<YASLC/ExpressionParser> Generated operation for reduction: '{}'", full_op);

        commands.push(full_op);

        // Create the combination expression
        let c = Expression::Combined(dest.clone());

        // Push the combination expression to the stack
        stack.push(c);

        Ok(commands)
    }

    /// Determines what the expression is and whether it should be inserted to the symbol table
    /// and/or stack as well as whether reduction should happen.
    /// TODO: Reduce this function call
    fn handle_expression(e: Expression, mut table: &mut SymbolTable, mut stack: &mut Vec<Expression>) -> Result<Option<Vec<String>>, String> {
        // Figure out what the expression
        match e.clone() {
            // The expression is an operand but may be an identifier (variable)
            // or a constant number
            Expression::Operand(t) => {
                // Check if it an identifier or constant number
                if t.is_type(TokenType::Identifier) {
                    // Check that the variable has been declared
                    if let Some(s) = table.get(&*t.lexeme()) {
                        match s.symbol_type {
                            SymbolType::Procedure => {
                                // Fail, we can't use procedures in expressions
                                panic!("Attempted to use a procedure as a variable in an expression!");
                            }
                            _ => {}
                        }
                        // Success, push the operand to the stack
                        stack.push(e);
                        return Ok(None);
                    } else {
                        panic!("Attempted to use variable '{}' that has not been declared!", t.lexeme());
                    }
                } else {
                    // It is a constant number, just push to the stack
                    stack.push(e);
                    return Ok(None);
                }

            },

            // The expression is an operator, we need to pop two operands and reduce them
            // to Expression::Combined.
            //
            // NOTE: This does not check for ordering because it is assumed the list of
            // expressions is already in postfix order.
            Expression::Operator(t_type) => {
                // Pop the previous two expressions and combine them
                let reduce_result = ExpressionParser::reduce_expression(t_type, &mut stack, &mut table);

                match reduce_result {
                    Ok(com) => return Ok(Some(com)),
                    Err(e) => panic!("Error while reducing expression stack: {}", e),
                }
            },
            Expression::Combined(_) => {
                stack.push(e);
                Ok(None)
            }
        }
    }

    /// Converts the vector of tokens to a vector of expressions and returns None if there was an
    /// invalid token.
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

    /// Converts the vector of expressions to postfix from infix.
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
                    log!("<YASLC/ExpressionParser> Found operator {}.", e);
                    // We have an operator, check it's precedence vs the top of the stack
                    if op_stack.len() != 0 {
                        while let Some(o) = op_stack.pop() {
                            // If its greater than current expression, pop and add to stack
                            if o > e {
                                log!("<YASLC/ExpressionParser> Pushing operator '{}' to the operand stack.", o);
                                stack.push(o);
                            } else {
                                op_stack.push(o);
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
        for e in stack.iter() {
            match e {
                &Expression::Operand(ref t) => log!("\t{:?}", t.lexeme()),
                &Expression::Operator(ref t) => log!("\t{}", t),
                _ => {},
            };
        }

        Some(stack)
    }
}
