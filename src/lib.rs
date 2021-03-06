mod cnf;
mod error;
mod symbol;
mod tree;

pub use cnf::*;
pub use error::*;
pub use symbol::*;
pub use tree::*;

use std::collections::HashMap;
use std::fmt::Debug;

fn rest_span(span: Span, len: usize) -> Option<Span> {
    if span.len() >= len {
        None
    } else {
        Some(Span::new(span.start() + span.len(), len - span.len()))
    }
}

pub trait Grammar {
    fn start_symbol(&self) -> Symbol;

    fn exist(&self, symbol: Symbol) -> bool;

    fn first(&self, symbol: Symbol) -> Option<HashSet<Symbol>>;

    fn follow(&self, symbol: Symbol) -> Option<HashSet<Symbol>>;

    fn derive(&self, base: Symbol, suffix: Symbol) -> Option<HashSet<Symbol>>;

    fn derive_single(&self, base: Symbol) -> Option<HashSet<Symbol>>;

    fn is_terminal(&self, input: Symbol) -> bool;

    fn is_non_terminal(&self, input: Symbol) -> bool;
}

#[derive(Clone, Debug)]
pub struct Parser<'a, G> {
    grammar: &'a G,
    src: &'a str,
    chars: Vec<char>,
    slices: HashMap<Span, Cell>,

    unknowns: Vec<Diagnostic>,
}

impl<'a, G: Grammar + Debug + Clone> Parser<'a, G> {
    pub fn new(grammar: &'a G) -> Self {
        Parser {
            grammar,
            src: "",
            chars: "".chars().collect(),
            slices: HashMap::new(),
            unknowns: vec![],
        }
    }

    pub fn parse(&mut self, string: &'a str) -> Result<&Node, Vec<Diagnostic>> {
        self.src = string;
        self.chars = string.chars().collect();
        self.slices = HashMap::new();
        self.unknowns = vec![];

        let src_len = self.src_len();
        for len in 1..(src_len + 1) {
            for span in self.spans_from_len(len) {
                self.parse_span(span);
            }
        }

        match self.get_cell(Span::new(0, src_len)) {
            Some(cell) => match cell.has(self.grammar.start_symbol()) {
                Some(node) => Ok(node),
                None => Err(self.unknowns.clone()),
            },
            None => Err(self.unknowns.clone()),
        }
    }

    fn parse_span(&mut self, span: Span) {
        match span.len() {
            1 => {
                let start = span.start();
                let c = *self.chars.get(start).unwrap();
                let cell = self.derive_char(span, c);
                if cell.is_empty() {
                    self.add_unknown(c, span)
                } else {
                    self.add_cell(cell, span)
                }
            }
            _ => {
                let mut cell_list: Vec<Cell> = Vec::new();
                for len in 1..span.len() {
                    let base_span = Span::new(span.start(), len);
                    if let Some(base_cell) = self.get_cell(base_span) {
                        if let Some(base_span) = base_cell.span() {
                            if let Some(rest_span) = rest_span(base_span, span.len()) {
                                if let Some(rest_cell) = self.get_cell(rest_span) {
                                    if let Some(next_cell) =
                                        self.derive(span, &base_cell, &rest_cell)
                                    {
                                        cell_list.push(next_cell);
                                    }
                                }
                            }
                        }
                    }
                }

                cell_list.sort();

                if let Some(cell) = cell_list.last() {
                    self.add_cell((*cell).clone(), span)
                }
            }
        }
    }

    fn derive_char(&self, span: Span, c: char) -> Cell {
        let mut next_cell = cell![;span];

        if let Some(symbol) = Symbol::from_char(c) {
            if let Some(symbols) = self.grammar.derive_single(symbol) {
                for sym in symbols {
                    next_cell.push_nodes(Node::new(
                        sym,
                        span,
                        NodeChildren::Single(Box::new(Node::new(symbol, span, NodeChildren::None))),
                    ))
                }
            }
        }

        next_cell
    }

    fn derive(&self, span: Span, base: &Cell, suffix: &Cell) -> Option<Cell> {
        let mut next_cell = cell![;span];

        for cur in &base.0 {
            for suffix in &suffix.0 {
                if let Some(symbols) = self.grammar.follow(cur.kind()) {
                    if symbols.iter().any(|&sym| sym.eq(&suffix.kind())) {
                        if let Some(symbols) = self.grammar.derive(cur.kind(), suffix.kind()) {
                            for symbol in symbols {
                                next_cell.push_nodes(Node::new(
                                    symbol,
                                    span,
                                    NodeChildren::Double(
                                        Box::new(cur.clone()),
                                        Box::new(suffix.clone()),
                                    ),
                                ))
                            }
                        }
                    }
                }
            }
        }

        if !next_cell.is_empty() {
            Some(next_cell)
        } else {
            None
        }
    }

    fn get_cell(&self, span: Span) -> Option<&Cell> {
        self.slices.get(&span)
    }

    fn spans_from_len(&self, len: usize) -> Vec<Span> {
        let mut spans: Vec<Span> = Vec::new();
        let src_len = self.src_len();

        debug_assert!(len <= src_len);

        for i in 0..(src_len - len + 1) {
            spans.push(Span::new(i, len))
        }

        spans
    }

    fn add_cell(&mut self, cell: Cell, span: Span) {
        self.slices.insert(span, cell);
    }

    fn add_unknown(&mut self, c: char, span: Span) {
        self.unknowns.push(Diagnostic::new(c, span));
    }

    fn src_len(&self) -> usize {
        self.chars.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
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
        let mut reader = Parser::new(&grammar);
        let result = reader.parse("3.51e+1");
        if let Ok(node) = result {
            assert_eq!(format!("{}", node), "3.51e+1");
        } else {
            panic!("Error")
        }
    }

    #[test]
    fn it_works_with_big() {
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
        let mut reader = Parser::new(&grammar);
        let result = reader.parse("3800909090.590900901e+1231231321");
        if let Ok(node) = result {
            assert_eq!(format!("{}", node), "3800909090.590900901e+1231231321");
        } else {
            panic!("Error")
        }
    }

    #[test]
    fn it_work_with_long() {
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
        let mut reader = Parser::new(&grammar);
        match reader.parse("12345678901234567890123456789012345678901234567890123456789012345678901234567890.12345678901234567890123456789012345678901234567890123456789012345678901234567890e+12345678901234567890123456789012345678901234567890123456789012345678901234567890") {
            Ok(node) => {
                assert_eq!(format!("{}", node), "12345678901234567890123456789012345678901234567890123456789012345678901234567890.12345678901234567890123456789012345678901234567890123456789012345678901234567890e+12345678901234567890123456789012345678901234567890123456789012345678901234567890");
            }
            Err(unknowns) => {
                panic!("Error")
            },
        }
    }
}
