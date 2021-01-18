extern crate cyk;

use cyk::cnf_grammar;

fn main() {
    let grammar = cnf_grammar! {
        // 3.51e+1
        // Number -> 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
        // Number -> Integer Digit
        // Number -> N1 Scale’ | Integer Fraction
        // N1 -> Integer Fraction
        // Integer -> 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
        // Integer -> Integer Digit
        // Fraction -> T1 Integer
        // T1 -> .
        // Scale’ -> N2 Integer
        // N2 -> T2 Sign
        // T2 -> e
        // Digit -> 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
        // Sign -> + | -
        Start("Number");
        NonTerminals[
            "Number", "N1", "Integer", "Fraction",
            "T1", "Scale", "N2", "T2", "Digit", "Sign"
        ];
        Terminals[
            "0", "1", "2", "3", "4", "5", "6",
            "7", "8", "9", ".", "e", "+", "-"
        ];
        Rules [
            "Number" => [
                ["Integer", "Digit"],
                ["N1", "Scale"],
                ["Integer", "Fraction"]
            ],
            "N1" => [
                ["Integer", "Fraction"]
            ],
            "Integer" => [
                ["Integer", "Digit"]
            ],
            "Fraction" => [
                ["T1", "Integer"]
            ],
            "Scale" => [
                ["N2", "Integer"]
            ],
            "N2" => [
                ["T2", "Sign"]
            ],
        ];
        TerminalRules [
            "Number" => [
                "0", "1", "2","3", "4", "5",
                "6", "7", "8", "9",
            ],
            "Integer" => [
                "0", "1", "2","3", "4", "5",
                "6", "7", "8", "9",
            ],
            "T1" => [
                "."
            ],
            "T2" => [
                "e"
            ],
            "Digit" => [
                "0", "1", "2","3", "4", "5",
                "6", "7", "8", "9",
            ],
            "Sign" => [
                "+", "-"
            ]
        ]
    };
    println!("{:?}", grammar);
    let mut reader = cyk::StringReader::new(&grammar);
    match reader.recognize("12345678901234567890.12345678901234567890e+12345678901234567890") {
        Ok(node) => {
            println!("Ok: {:?}", node);
            println!("Ok: {}", node);
        }
        Err(unknowns) => unknowns.iter().for_each(|item| {
            println!("Err: {:?}", item);
        }),
    }
}
