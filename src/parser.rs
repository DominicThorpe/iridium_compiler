extern crate pest;


use pest::Parser;
use pest::error::Error;


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CParser;


#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negation,
    Complement,
    LogicalNeg,
}


#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide
}


#[derive(Debug, Clone)]
pub enum ASTNode {
    Integer(i16),
    ReturnStatement(Box<ASTNode>),
    Factor {
        literal: Option<Box<ASTNode>>,
        operator: Option<UnaryOperator>, 
        expression: Option<Box<ASTNode>>,
    },
    Term {
        lhs: Box<ASTNode>,
        operator: Option<BinaryOperator>,
        rhs: Option<Box<ASTNode>>
    },
    Expression {
        lhs: Box<ASTNode>,
        operator: Option<BinaryOperator>,
        rhs: Option<Box<ASTNode>>
    },
    Function {
        return_type: String,
        identifier: String,
        statements: Vec<ASTNode>
    },
}


fn convert_str_to_int(val:&str) -> i16 {
    if val.starts_with("0b") {
        i64::from_str_radix(&val[2..], 2).unwrap().try_into().unwrap()
    } else if val.starts_with("0x") {
        i64::from_str_radix(&val[2..], 16).unwrap().try_into().unwrap()
    } else {
        str::parse::<i16>(val).unwrap().try_into().unwrap()
    }
}


fn get_node_from_unary_operator(operator:&str) -> UnaryOperator {
    match operator {
        "-" => UnaryOperator::Negation,
        "!" => UnaryOperator::LogicalNeg,
        "~" => UnaryOperator::Complement,
        _ => panic!("{} is not a recongised unary operator!", operator)
    }
}


fn get_node_from_binary_operator(operator:&str) -> BinaryOperator {
    match operator {
        "+" => BinaryOperator::Add,
        "-" => BinaryOperator::Subtract,
        "*" => BinaryOperator::Multiply,
        "/" => BinaryOperator::Divide,
        _ => panic!("{} is not a recongised binary operator!", operator)
    }
}


fn build_ast_from_factor(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    match pair.as_rule() {
        Rule::factor => {
            let mut pair = pair.into_inner();
            let token = pair.next().unwrap();
            match token.as_rule() {
                Rule::unary_operator => {
                    let operator = get_node_from_unary_operator(token.as_str());
                    let next = pair.next().unwrap();
                    let sub_expression = build_ast_from_expression(&mut pair, next, None);
                    ASTNode::Factor {
                        operator: Some(operator),
                        expression: Some(Box::new(sub_expression)),
                        literal: None
                    }
                },

                Rule::number=> {
                    let literal = ASTNode::Integer(convert_str_to_int(token.as_str()));
                    ASTNode::Factor {
                        operator: None,
                        expression: None,
                        literal: Some(Box::new(literal))
                    }
                },

                unknown_expr => panic!("Unexpected token: {:?}", unknown_expr),
            }
        },

        unknown_expr => panic!("Unexpected factor: {:?}", unknown_expr),
    }
}


fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    match pair.as_rule() {
        Rule::term => {
            let term = pair.into_inner().next().unwrap();
            ASTNode::Term {
                lhs: Box::new(build_ast_from_factor(term)),
                operator: None,
                rhs: None
            }
        },

        unknown_expr => panic!("Unexpected term: {:?}", unknown_expr),
    }
}


fn build_ast_from_expression(parent:&mut pest::iterators::Pairs<Rule>, pair: pest::iterators::Pair<Rule>, sub_expr:Option<Box<ASTNode>>) -> ASTNode {
    match pair.as_rule() {
        Rule::expression => {
            let mut parent = pair.into_inner();
            let pair = parent.next().unwrap();
            build_ast_from_expression(&mut parent, pair, None)
        },

        Rule::term => {
            match parent.peek() {
                Some(token) => {
                    match token.as_rule() {
                        Rule::add_or_sub => {
                            let next_op_token = parent.next().unwrap();
                            let next_int_token = parent.next().unwrap();
                            let operator = get_node_from_binary_operator(next_op_token.as_str());
                            let sub = ASTNode::Expression {
                                lhs: sub_expr.unwrap_or(Box::new(build_ast_from_term(pair))),
                                operator: Some(operator),
                                rhs: Some(Box::new(build_ast_from_term(next_int_token.clone())))
                            };

                            build_ast_from_expression(parent, next_int_token, Some(Box::new(sub)))
                        },

                        unknown_expr => panic!("Unexpected token: {:?}", unknown_expr),
                    }
                },

                None => match sub_expr { // last term in the expression
                    Some(s) => *s,
                    None => build_ast_from_term(pair)
                } 
            }
        },

        unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
    }
}


fn build_ast_from_stmt(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    match pair.as_rule() {
        Rule::return_stmt => {
            let mut pair = pair.into_inner();
            let expression = pair.next().unwrap();
            ASTNode::ReturnStatement(Box::new(build_ast_from_expression(&mut pair, expression, None)))
        },
        unknown_expr => panic!("Unexpected statement construct: {:?}", unknown_expr),
    }
}


pub fn build_ast_from_toplvl(pair: pest::iterators::Pair<Rule>) -> ASTNode {
    match pair.as_rule() {
        Rule::function => {
            let mut pair = pair.into_inner();
            let datatype = pair.next().unwrap().as_str().to_owned();
            let identifier = pair.next().unwrap().as_str().to_owned();
            let mut statements:Vec<ASTNode> = Vec::new();
            
            loop {
                match pair.next() {
                    Some(statement) => {
                        statements.push(build_ast_from_stmt(statement));
                    },

                    None => {
                        break;
                    }
                }
            }

            ASTNode::Function {
                return_type: datatype, 
                identifier: identifier, 
                statements: statements
            }
        },

        unknown_expr => panic!("Unexpected toplevel construct: {:?}", unknown_expr),
    }
}


pub fn parse(source: &str) -> Result<Vec<ASTNode>, Error<Rule>> {
    let mut ast = vec![];

    let pairs = CParser::parse(Rule::program, source)?;
    // println!("{:#?}", pairs);
    for pair in pairs {
        match pair.as_rule() {
            Rule::function => {
                ast.push(build_ast_from_toplvl(pair));
            }
            _ => {}
        }
    }

    Ok(ast)
}


#[cfg(test)]
mod tests {
    use std::fs::read_dir;

    extern crate pest;
    use pest::Parser;

    use crate::parser;

    #[derive(Parser)]
    #[grammar = "grammar.pest"]
    pub struct CParser;

    #[test]
    fn test_empty_program() {
        let _parsed = CParser::parse(Rule::program, "int main() {}").unwrap();
    }

    #[test]
    fn test_return_decimal_program() {
        let _parsed = CParser::parse(Rule::program, "int main() {
            return 55;
        }").unwrap();
    }

    #[test]
    fn test_return_hex_program() {
        let _parsed = CParser::parse(Rule::program, "int main() {
            return 0xEf7;
        }").unwrap();
    }

    #[test]
    fn test_basic_valid_return_programs() {
        let _parsed = CParser::parse(Rule::program, "int main() {
            return 0b110011;
        }").unwrap();
    }

    #[test]
    fn test_parse_basic_program() {
        let valid_prog_names:Vec<_> = read_dir("test_programs/return_int_literal/valid")
                            .unwrap()
                            .map(|dir| dir.unwrap().path())
                            .into_iter()
                            .collect();
        for path in valid_prog_names {
            let unparsed_file = std::fs::read_to_string(path.clone()).expect(&format!("Cannot read C file {:?}", path));
            let ast = parser::parse(&unparsed_file).unwrap();
            match ast[0].clone() {
                parser::ASTNode::Function { return_type, identifier, statements } => {
                    assert_eq!(return_type, "int");
                    assert_eq!(identifier, "main");
                    assert_eq!(statements.len(), 1);
                },

                _ => panic!("Node {:?} should be a function", ast[0])
            };
        }
    }

    #[test]
    fn test_basic_invalid_return_programs() {
        let valid_prog_names:Vec<_> = read_dir("test_programs/return_int_literal/invalid")
                            .unwrap()
                            .map(|dir| dir.unwrap().path())
                            .into_iter()
                            .collect();
        for path in valid_prog_names {
            let unparsed_file = std::fs::read_to_string(path.clone()).expect(&format!("Cannot read C file {:?}", path));
            match parser::parse(&unparsed_file) {
                Ok(_) => panic!("Invalid C file {} should fail but did not!", path.display()),
                Err(_) => {}
            }
        }
    }

    #[test]
    fn test_parse_valid_unary_op_program() {
        let valid_prog_names:Vec<_> = read_dir("test_programs/unary_operators/valid")
                            .unwrap()
                            .map(|dir| dir.unwrap().path())
                            .into_iter()
                            .collect();
        for path in valid_prog_names {
            let unparsed_file = std::fs::read_to_string(path.clone()).expect(&format!("Cannot read C file {:?}", path));
            let _ = parser::parse(&unparsed_file).unwrap();
        }
    }

    #[test]
    fn test_parse_invalid_unary_op_program() {
        let valid_prog_names:Vec<_> = read_dir("test_programs/unary_operators/invalid")
                            .unwrap()
                            .map(|dir| dir.unwrap().path())
                            .into_iter()
                            .collect();
        for path in valid_prog_names {
            let unparsed_file = std::fs::read_to_string(path.clone()).expect(&format!("Cannot read C file {:?}", path));
            match parser::parse(&unparsed_file) {
                Ok(_) => panic!("Invalid C file {} should fail but did not!", path.display()),
                Err(_) => {}
            }
        }
    }
}
