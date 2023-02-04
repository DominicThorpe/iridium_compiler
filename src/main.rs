mod lexer;

#[macro_use]
extern crate pest_derive;


fn main() {
    lexer::parse();
}
