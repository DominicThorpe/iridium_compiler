extern crate pest;


use pest::Parser;


#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CParser;


pub fn parse() {
    let successful_parse = CParser::parse(Rule::number, "273");
    println!("{:#?}", successful_parse);

    let unsuccessful_parse = CParser::parse(Rule::number, "this is not a number");
    println!("{:?}", unsuccessful_parse);
}


#[cfg(test)]
mod tests {
    extern crate pest;
    use pest::Parser;

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
    fn test_return_binary_program() {
        let _parsed = CParser::parse(Rule::program, "int main() {
            return 0b110011;
        }").unwrap();
    }
}
