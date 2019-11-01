mod parser;

use parser::parse_lisp_expr;

fn main() {
    let output = parse_lisp_expr("(a '(quoted (dotted special . list)) test)");
    println!("Output is {:?}", output);
}


