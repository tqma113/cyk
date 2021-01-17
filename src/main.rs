extern crate cyk;

use cyk::cnf;

fn main() {
    let parser = cnf! {
        Start("Number");
        NonTerminal[
            "Number",
            "N1",
            "Integer",
            "Fraction",
            "T1",
            "Scale",
            "N2",
            "T2",
            "Digit",
            "Sign"
        ];
        Terminal[
            "0",
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
            ".",
            "e",
            "+",
            "-"
        ];
        Rules [
            "Number" => [
                ["0"],
                ["1"],
                ["2"],
                ["3"],
                ["4"],
                ["5"],
                ["6"],
                ["7"],
                ["8"],
                ["9"],
                ["Integer", "Digit"],
                ["N1", "Scale"],
                ["Integer", "Fraction"]
            ],
            "N1" => [
                ["Integer", "Fraction"]
            ],
            "Integer" => [
                ["0"],
                ["1"],
                ["2"],
                ["3"],
                ["4"],
                ["5"],
                ["6"],
                ["7"],
                ["8"],
                ["9"],
                ["Integer", "Digit"]
            ],
            "Fraction" => [
                ["T1", "Integer"]
            ],
            "T1" => [
                ["."]
            ],
            "Scale" => [
                ["N2", "Integer"]
            ],
            "N2" => [
                ["T2", "Sign"]
            ],
            "T2" => [
                ["e"]
            ],
            "Digit" => [
                ["0"],
                ["1"],
                ["2"],
                ["3"],
                ["4"],
                ["5"],
                ["6"],
                ["7"],
                ["8"],
                ["9"]
            ]
        ]
    };
    println!("{:?}", parser);
}
