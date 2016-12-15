/// parser/expression.rs
///
/// The expression module contains the expression parser and all the implementation
/// for the expression parser.

#[cfg(test)]
mod tests;

pub use super::{Token, TokenType, KeywordType};
pub use super::{Symbol, SymbolTable, SymbolType, SymbolValueType};
use super::CommandBuilder;

use std::cmp::Ordering;
use std::fmt;

/// Set to true if you want the expression parser to print its process.
static mut VERBOSE: bool = false;

macro_rules! log {
    ($message:expr $(,$arg:expr)*) => {
        unsafe {
            if VERBOSE == true {
                println!($message, $($arg,)*);
            }
        }
    };

    (NNL $message:expr $(,$arg:expr)*) => {
        unsafe {
            if VERBOSE == true {
                print!($message, $($arg,)*);
            }
        }
    };
}

// Helper function
pub fn type_for_string(l: &String) -> Option<SymbolValueType> {
    // If the lexeme is numeric it's a number, otherwise if its "true"/"false its a boolean"
    // if its neither then crash
    match l.parse::<i32>() {
        Ok(_) => {
            // Its a number
            log!("Determined that the type for string {} is int.", l);
            Some(SymbolValueType::Int)
        },
        Err(_) => {
            // It is not a number, check if it is a boolean
            if l == "true" {
                log!("Determined that the type for string {} is bool.", l);
                Some(SymbolValueType::Bool)
            } else if l == "false" {
                log!("Determined that the type for string {} is bool.", l);
                Some(SymbolValueType::Bool)
            } else {
                // We don't know what it is, crash.
                println!("<YASLC/ExpressionParser> Warning: unable to identify value type for token {}.", l);
                None
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
enum OType {
    // String is the value of the variable
    Static(String),

    // String is the name of the varible and symbol
    Variable(String)
}

/// Expression represents a single piece of expressions.
#[derive(Eq, PartialEq, Clone)]
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
    Operand(OType),

    /// A combined expression using three other expressions, in
    /// operand - operator - operand format.
    Combined(Symbol),
}

impl Expression {
    /// Creates a new expression from a token and returns Some(e) where e is a valid expression,
    /// or returns None if the expression is not valid given the token.
    fn from_token(t: Token) -> Option<Expression> {
        match t.token_type() {
            // Constant numbers
            TokenType::Number => Some(Expression::Operand(OType::Static(t.lexeme()))),

            // Operators
            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Keyword(KeywordType::Div)
            | TokenType::Keyword(KeywordType::Mod) | TokenType::GreaterThan | TokenType::LessThan
            | TokenType::GreaterThanOrEqual | TokenType::LessThanOrEqual | TokenType::EqualTo
            | TokenType::NotEqualTo => Some(Expression::Operator(t.token_type())),

            // Boolean exclusive operators
            TokenType::Keyword(KeywordType::And) | TokenType::Keyword(KeywordType::Or)
                => Some(Expression::Operator(t.token_type())),

            // Variables and Constants
            TokenType::Identifier => Some(Expression::Operand(OType::Variable(t.lexeme()))),

            // true and false
            TokenType::Keyword(KeywordType::True) => Some(Expression::Operand(OType::Static(format!("true")))),
            TokenType::Keyword(KeywordType::False) => Some(Expression::Operand(OType::Static(format!("false")))),

            _ => None,
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

                    &Operator(TokenType::Keyword(KeywordType::And)) | &Operator(TokenType::Keyword(KeywordType::Or))
                        => Some(Ordering::Greater),

                    &Operand(_) => Some(Ordering::Greater),

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

                    &Operator(TokenType::Keyword(KeywordType::And)) | &Operator(TokenType::Keyword(KeywordType::Or))
                        => Some(Ordering::Greater),

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

                    &Operator(TokenType::Keyword(KeywordType::And)) | &Operator(TokenType::Keyword(KeywordType::Or))
                        => Some(Ordering::Greater),

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

                    &Operator(TokenType::Keyword(KeywordType::And)) | &Operator(TokenType::Keyword(KeywordType::Or))
                        => Some(Ordering::Greater),

                    _ => {
                        None
                    },
                }
            }

            &Operator(TokenType::Keyword(KeywordType::And)) | &Operator(TokenType::Keyword(KeywordType::Or)) => Some(Ordering::Less),

            &Operand(_) => {
                // Any number
                match other {
                    // &Operator(TokenType::Plus) | &Operator(TokenType::Minus) |
                    // &Operator(TokenType::Star) | &Operator(TokenType::Keyword(KeywordType::Div))|
                    // &Operator(TokenType::Keyword(KeywordType::Mod)) => Some(Ordering::Greater),
                    &Operator(_) => Some(Ordering::Greater),

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
                write!(f, "<Expr: Operator, {}>", t)
            },
            &Expression::Operand(ref v) => {
                match v {
                    &OType::Variable(ref t) => write!(f, "<Expr: Operand, {}>", t),
                    &OType::Static(ref l) => write!(f, "<Expr: StaticOperand, {}>", l),
                }
            },
            &Expression::Combined(ref s) => {
                write!(f, "<Expr: Combined, {:?}>", s)
            },
        }
    }
}

/// ExpressionParser validates the syntax of an expression as well as reduces it and
/// manages memory allocation for temporary variables used for arithmatic.
pub struct ExpressionParser {
    /// The list of commands to be pushed onto the program given this expression.
    commands: CommandBuilder,

    expressions: Vec<Expression>,

    // The stack used when reducing the expression to one symbol
    stack: Vec<Expression>,

    table: SymbolTable,
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

        Some(ExpressionParser {
            commands: CommandBuilder::new(),
            expressions: postfix_exp,
            stack: Vec::<Expression>::new(),
            table: table,
        })
    }

    pub fn parse(mut self) -> Result<(Symbol, CommandBuilder), String> {
        if self.expressions.len() == 1 {
            match self.expressions.remove(0) {
                Expression::Operand(o_type) => {
                    match o_type {
                        OType::Variable(l) => {
                            let f_symbol = self.table.get(&*l).unwrap();
                            return Ok((f_symbol.clone(), self.commands));
                            // self.commands.push_command(format!("movw "))
                        },
                        OType::Static(l) => {
                            let t = self.table.temp(SymbolType::Constant(type_for_string(&l).unwrap()));
                            self.commands.push_command(format!("movw #{} +0@R1", l));
                            return Ok((t, self.commands));
                        }
                    }
                },
                _ => {},
            };
        }

        let f_symbol = match self.reduce_expression_stack() {
            Some(s) => s,
            None => {
                return Err(format!("<YASLC/ExpressionParser> Attempted to get final symbol for expression but none was found!"));
            }
        };

        // // Now that we have one single expression, move it to the SP
        let sp_mov = format!("movw {} +0@R1", f_symbol.location());
        self.push_command(sp_mov);

        Ok((f_symbol, self.commands))
    }

    fn push_command(&mut self, command: String) {
        log!("Pushing command: {}", command);
        self.commands.push_command(command);
    }

    /// Reduces the stack of postfix expressions until there is only one remaining.
    fn reduce_expression_stack(&mut self) -> Option<Symbol> {
        // Move the register up by 1
        self.table.up_register();

        // Reduce the list until there are no commands remaining
        while self.expressions.len() > 0 {
            log!(NNL "Reducing in state:\n\tExpressions:[ ");
            for e in self.expressions.iter() {
                log!(NNL "{}, ", e);
            }
            log!(NNL "],\n\tStack: [");
            for e in self.stack.iter() {
                log!(NNL "{}, ", e);
            }
            log!("]");

            // Pop the first expression
            let e = self.expressions.remove(0);

            match self.handle_expression(e) {
                Ok(_) => {},
                Err(error) => {
                    println!("<YASLC/ExpressionParser> Error handling expression: {}", error);
                    return None;
                }
            };

        }

        let f_symbol = match self.f_symbol() {
            Some(s) => {
                s
            },
            _ => {
                println!("<YASLC/ExpressionParser> Error: Expected to find final symbol in expression parser but none was found!");
                return None;
            }
        };

        Some(f_symbol)
    }

    /// Returns the final symbol, given the stack is reduced
    fn f_symbol(&mut self) -> Option<Symbol> {
        if self.stack.len() == 0 {
            panic!("<YASLC/ExpressionParser> Internal error attempted to get the final symbol of an expression but it was not found!");
        }

        match self.stack.remove(0) {
            // A combined expression
            Expression::Combined(s) => Some(s),

            // A single operand
            Expression::Operand(o_type) => {
                match o_type {
                    OType::Static(l) => {
                        let s = self.table.temp(SymbolType::Variable(type_for_string(&l).unwrap()));
                        self.push_command(format!("movw #{} {}", l, s.location()));
                        Some(s.clone())
                    },
                    OType::Variable(t) => {
                        let symbol = match self.table.get(&*t) {
                            Some(s) => s,
                            None => {
                                panic!("<YASLC/ExpressionParser> Attempted to use a symbol that was not found in the symbol table! This is very unexpected...");
                            }
                        };
                        Some(symbol.clone())
                    }
                }
            }
            _ => {
                panic!("Found expression consisting of only an operator!");
            }
        }
    }

    fn last_two_expressions(mut stack: &mut Vec<Expression>) -> Result<(Expression, Expression), String> {
        let e2 = match stack.pop() {
            Some(s) => s,
            None => {
                return Err(format!("Attempted to reduce expression but there is two missing operands!"));
            }
        };
        let e1 = match stack.pop() {
            Some(s) => s,
            None => {
                return Err(format!("Attempted to reduce expression but there is a missing operand!"));
            }
        };
        Ok((e1, e2))
    }

    /// Reduces the previous two expressions on self.stack with the token type t_type
    fn reduce_expression(&mut self, t_type: TokenType) -> Result<(), String> {
        // Pop the previous two expressions
        let (e1, e2) = match ExpressionParser::last_two_expressions(&mut self.stack) {
            Ok((r1, r2)) => (r1, r2),
            Err(e) => panic!("<YASLC/ExpresionParser> {}", e),
        };

        log!("<YASLC/ExpressionParser> Reducing expressions {} and {} using {}.", e1, e2, t_type);

        // Match the first expression because if it is a temp variable we can operate on that
        // and not have to create another temp variable
        let s1 = match e1 {
            Expression::Operand(o_type) => {
                match o_type {
                    // If its a variable
                    OType::Variable(l) => {
                        match self.table.get(&*l) {
                            Some(x) => x.clone(),
                            None => panic!("Attempted to use variable '{}' that has not been declared!", l),
                        }
                    },

                    // It is a constant, initialize to a temp
                    OType::Static(l) => {
                        let temp = self.table.temp(SymbolType::Variable(type_for_string(&l).unwrap()));
                        self.push_command(format!("movw #{} {}", l, temp.location()));
                        temp
                    }
                }
            },
            Expression::Combined(s) => s,
            _ => panic!("Found an operator where we were expecting an operand!"),
        };

        let s2 = match e2 {
            Expression::Operand(o_type) => {
                match o_type {
                    // If its a variable
                    OType::Variable(l) => {
                        match self.table.get(&*l) {
                            Some(x) => x.clone(),
                            None => panic!("Attempted to use variable '{}' that has not been declared!", l),
                        }
                    },

                    // It is a constant, initialize to a temp
                    OType::Static(l) => {
                        let temp = self.table.temp(SymbolType::Variable(type_for_string(&l).unwrap()));
                        self.push_command(format!("movw #{} {}", l, temp.location()));
                        temp
                    }
                }
            },
            Expression::Combined(s) => s,
            _ => panic!("Found an operator where we were expecting an operand!"),
        };

        match &s1.symbol_type {
            &SymbolType::Variable(ref v1) | &SymbolType::Constant(ref v1) => {
                match &s2.symbol_type {
                    &SymbolType::Variable(ref v2) | &SymbolType::Constant(ref v2) => {
                        if v1 != v2 {
                            log!("s1: {:?}, s2: {:?}", s1.symbol_type, s2.symbol_type);
                            return Err(format!("<YASLC/ExpressionParser> Attempted to perform operation on two symbols which don't have the same type!"));
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        };

        // Find the destination symbol
        let mut dest = if s1.is_temp() {
            // We can operate on s1
            log!("We can operate on {:?} for expression in place of a temp because it is already a temp!", s1);
            s1.clone()
        } else {
            // We have to operate on a temp
            //
            // Move the value from the first symbol to temp
            let temp = self.table.temp(s1.symbol_type.clone());
            log!("Generated temp symbol {:?} for expression.", temp);
            let mov = format!("movw {} {}", s1.location(), temp.location());
            self.push_command(mov);
            temp
        };

        // Determine the operator string given the token type
        let op = match t_type {
            TokenType::Plus => "addw",
            TokenType::Minus => "subw",
            TokenType::Star => "mulw",
            TokenType::Keyword(KeywordType::Div) => "divw",
            TokenType::Keyword(KeywordType::Mod) => {
                // Special case, will return value for the function
                log!("Reducing using Mod and special commands for that.");

                // Generate temp 1 and 2
                let temp1 = dest;
                let temp2 = self.table.temp(s2.symbol_type.clone());

                // Divide temp1 by s2
                self.push_command(format!("divw {} {}", s2.location(), temp1.location()));

                // Multiply temp1 by s2
                self.push_command(format!("mulw {} {}", s2.location(), temp1.location()));

                // Move s1 to temp2
                self.push_command(format!("movw {} {}", s1.location(), temp2.location()));

                // Subtract temp1 from temp2
                self.push_command(format!("subw {} {}", temp1.location(), temp2.location()));

                // Move temp2 to s1 (dest)
                self.push_command(format!("movw {} {}", temp2.location(), temp1.location()));

                // Generate the combined expression
                let c = Expression::Combined(temp1);
                log!("<YASLC/ExpressionParser> Successfully generated 'mod' expression code, {}", c);
                self.stack.push(c);

                return Ok(());
            },

            TokenType::GreaterThan | TokenType::LessThan | TokenType::GreaterThanOrEqual
            | TokenType::LessThanOrEqual | TokenType::EqualTo | TokenType::NotEqualTo  => {
                log!("Reducing using a boolean expression.");

                // if we have == or <> check that it is NOT boolean type
                if t_type == TokenType::EqualTo || t_type == TokenType::NotEqualTo {
                    let vt = match s1.symbol_type() {
                        &SymbolType::Variable(ref vt) | &SymbolType::Constant(ref vt) => {
                            vt
                        },
                        _ => return Err(format!("<YASLC/ExpressionParser> Found an error that should have been caught a long time ago...")),
                    };
                    // If its a boolean, return an error
                    match vt {
                        &SymbolValueType::Bool => return Err(format!("Expected symbol {:?} to be an integer but it was a boolean!", s1)),
                        _ => {},
                    };
                }

                // Get the comparator command
                let comp  = match t_type {
                    TokenType::GreaterThan => "bgtr",
                    TokenType::GreaterThanOrEqual => "bgeq",
                    TokenType::EqualTo => "beq",
                    TokenType::NotEqualTo => "bneq",
                    TokenType::LessThanOrEqual => "bleq",
                    TokenType::LessThan => "blss",
                    _ => panic!(),
                };

                let bool_temp = self.table.bool_temp();

                // We don't need to type check for comparison because both are stored as integers
                self.push_command(format!("cmpw {} {}", s1.location(), s2.location()));
                self.push_command(format!("{} $b_true{}", comp, bool_temp));
                self.push_command(format!("movw #0 {}", dest.location()));
                self.push_command(format!("jmp $b_end{}", bool_temp));
                self.push_command(format!("$b_true{} movw #1 {}", bool_temp, dest.location()));
                self.commands.set_prefix(format!("$b_end{}", bool_temp));

                // Change the value type because all of these comparisons create a boolean
                dest.set_value_type(SymbolValueType::Bool);
                // Create the combination expression
                // Push the combination expression to the stack
                let c = Expression::Combined(dest);
                self.stack.push(c);

                return Ok(());
            },

            TokenType::Keyword(KeywordType::And) | TokenType::Keyword(KeywordType::Or) => {
                log!("Reducing using 'and/or' special case.");
                let vt = match s1.symbol_type() {
                    &SymbolType::Variable(ref vt) | &SymbolType::Constant(ref vt) => {
                        vt
                    },
                    _ => return Err(format!("<YASLC/ExpressionParser> Found an error that should have been caught a long time ago...")),
                };
                // If its an integer, return an error
                match vt {
                    &SymbolValueType::Int => return Err(format!("Expected symbol {:?} to be an boolean but it was a integer!", s1)),
                    _ => {},
                };


                // For OR expressions we exit if either is TRUE and set to TRUE so we can exit
                // early.
                // For AND expressions we exit if either is NOT TRUE and set to NOT TRUE so
                // we can exit early.
                //
                // o1 => symbol is compared to and set to if both are equal to it
                // o2 => the alternate if either is not equal to

                let (o1, o2) = match t_type {
                    TokenType::Keyword(KeywordType::And) => ("1", "0"),
                    TokenType::Keyword(KeywordType::Or) => ("0", "1"),
                    _ => panic!(),
                };

                let bool_temp = self.table.bool_temp();

                self.push_command(format!("cmpw {} {}", s1.location(), o1));
                self.push_command(format!("bneq $b_else{}", bool_temp));
                self.push_command(format!("cmpw {} {}", s2.location(), o1));
                self.push_command(format!("bneq $b_else{}", bool_temp));
                self.push_command(format!("movw {} {}", o1, dest.location()));
                self.push_command(format!("jmp $b_end{}", bool_temp));
                self.push_command(format!("movw {} {}", o2, dest.location()));
                self.commands.set_prefix(format!("$b_end{}", bool_temp));

                // Change the value type because all of these comparisons create a boolean
                dest.set_value_type(SymbolValueType::Bool);
                // Create the combination expression
                // Push the combination expression to the stack
                let c = Expression::Combined(dest);
                self.stack.push(c);

                return Ok(());
            }

            n => {
                panic!("Unrecognized operator '{}' in expression!", n)
            },
        };

        // Push the combination expression to the stack
        let c = Expression::Combined(dest.clone());
        log!("Got the combined expression {}", c);
        self.stack.push(c);

        // Perform the operation
        let full_op = format!("{} {} {}", op, s2.location(), dest.location());

        log!("<YASLC/ExpressionParser> Generated operation for reduction: '{}'", full_op);

        self.push_command(full_op);

        Ok(())
    }

    /// Determines what the expression is and whether it should be inserted to the symbol table
    /// and/or stack as well as whether reduction should happen.
    fn handle_expression(&mut self, e: Expression) -> Result<(), String> {
        // Figure out what the expression
        match e.clone() {
            // The expression is an operand but may be an identifier (variable)
            // or a constant number
            Expression::Operand(o_type) => {
                // Check if it an identifier or constant number
                match o_type {
                    OType::Variable(l) => {
                        // Check that the variable has been declared
                        if let Some(s) = self.table.get(&*l) {
                            match s.symbol_type {
                                SymbolType::Procedure => {
                                    // Fail, we can't use procedures in expressions
                                    panic!("Attempted to use a procedure as a variable in an expression!");
                                }
                                _ => {}
                            }
                            // Success, push the operand to the stack
                            self.stack.push(e);
                            return Ok(());
                        } else {
                            panic!("Attempted to use variable '{}' that has not been declared!", l);
                        }
                    },
                    OType::Static(_) => {
                        // It is a constant number, just push to the stack
                        self.stack.push(e);
                        return Ok(());
                    }
                }
            },

            // The expression is an operator, we need to pop two operands and reduce them
            // to Expression::Combined.
            //
            // NOTE: This does not check for ordering because it is assumed the list of
            // expressions is already in postfix order.
            Expression::Operator(t_type) => {
                // Pop the previous two expressions and combine them
                let reduce_result = self.reduce_expression(t_type);

                match reduce_result {
                    Ok(_) => Ok(()),
                    Err(e) => return Err(format!("Error while reducing expression stack: {}", e)),
                }
            },
            Expression::Combined(_) => {
                self.stack.push(e);
                Ok(())
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

            log!("<YASLC/ExpressionParser> Popped token for conversion to expression: {}", t);

            // Attempt to convert it to an expression
            if let Some(e) = Expression::from_token(t.clone()) {
                expressions.push(e);
            } else {
                // Converting to an expression failed likely because it is an invalid token
                // Stop all conversion and return
                println!("<YASLC/ExpressionParser> Error: invalid token {} in expression.", t);
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

        while let Some(o) = op_stack.pop() {
            stack.push(o);
        }

        log!("<YASLC/ExpressionParser> Successfully converted infix expressions to postfix.");
        log!(NNL "[");
        for e in stack.iter() {
            match e {
                &Expression::Operand(ref t) => match t{
                    &OType::Static(ref l) | &OType::Variable(ref l) => log!(NNL "{}, ", l),
                },
                &Expression::Operator(ref t) => log!(NNL "{}, ", t),
                _ => {},
            };
        }
        log!("]");

        Some(stack)
    }
}
