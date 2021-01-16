extern crate cyk;

use cyk::cyk;

fn main() {
    let parser = cyk!{
        Start("A");
        NonTerminal["A","B"];
        Terminal["a","b","c","d"];
        Rules [
            "A" => [["a"]]
        ]
    };
    println!("{:?}", parser);
}
