mod parser;

#[macro_use]
extern crate pest_derive;


fn main() {
    let unparsed_file = std::fs::read_to_string("test_programs/unary_operators/valid/nested_ops.c").expect("cannot read C file");
    let ast = parser::parse(&unparsed_file).unwrap();
    println!("{:#?}", ast);
}
