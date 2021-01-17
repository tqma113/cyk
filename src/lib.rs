mod cnf;
mod symbol;
mod tree;
mod error;

pub use cnf::*;
pub use symbol::*;
pub use tree::*;
pub use error::*;

use std::collections::HashMap;
use std::fmt::Debug;

fn rest_span(span: Span, len: usize) -> Option<Span> {
    if span.len() >= len {
        return None;
    } else {
        Some(Span::new(span.start() + span.len(), len - span.len()))
    }
}

pub trait Grammar {
    fn start_symbol(self) -> Symbol;

    fn exist(self, symbol: Symbol) -> bool;

    fn first(self, symbol: Symbol) -> Option<Vec<Symbol>>;

    fn follow(self, symbol: Symbol) -> Option<Vec<Symbol>>;

    fn derive(self, base: Symbol, suffix: Symbol) -> Option<Vec<Symbol>>;

    fn derive_single(self, base: Symbol) -> Option<Vec<Symbol>>;
}

#[derive(Clone, Debug)]
pub struct StringReader<'a, G> {
    grammar: &'a G,
    src: &'a str,
    chars: Vec<char>,
    slices: HashMap<Span, Cell>,

    unknowns: Vec<Diagnostic>,
}

impl<'a, G: Grammar + Debug + Clone> StringReader<'a, G> {
    pub fn new(grammar: &'a G) -> Self {
        StringReader {
            grammar,
            src: "",
            chars: "".chars().collect(),
            slices: HashMap::new(),
            unknowns: vec![],
        }
    }

    pub fn recognize(&mut self, string: &'a str) -> Result<Node, Vec<Diagnostic>> {
        self.src = string;
        self.chars = string.chars().collect();
        self.slices = HashMap::new();
        self.unknowns = vec![];

        let src_len = self.clone().src_len();
        for len in 1..(src_len + 1) {
            for span in self.clone().spans_from_len(len) {
                self.recognize_span(span);
            }
        }

        match self.get_cell(Span::new(0, src_len)) {
            Some(cell) => {
                match cell.clone().has(self.grammar.clone().start_symbol()) {
                    Some(node) => Ok(node.clone()),
                    None => {
                        println!("span: {:?}", cell);
                        Err(self.unknowns.clone())
                    },
                }
            }
            None => Err(self.unknowns.clone()),
        }
    }

    fn recognize_span(&mut self, span: Span) {
        match span.len() {
            1 => {
                let start = span.start();
                let c = *self.chars.get(start).unwrap();
                let cell = self.clone().derive_char(span, c);
                if cell.clone().len() == 0 {
                    self.add_unknown(c, span)
                } else {
                    self.add_cell(cell.clone(), span)
                }
            }
            _ => {
                let mut cell_list: Vec<Cell> = Vec::new();
                for len in 1..span.len() {
                    let base_span = Span::new(span.start(), len);
                    if let Some(base_cell) = self.get_cell(base_span) {
                        if let Some(base_span) =  base_cell.clone().span() {
                            if let Some(rest_span) = rest_span(base_span, span.len()) {
                                if let Some(rest_cell) = self.get_cell(rest_span) {
                                    let next_cell = self.clone().derive(span, base_cell, rest_cell);
                                    cell_list.push(next_cell.clone());
                                }
                            }
                        }
                    }
                }

                cell_list.sort();

                match cell_list.last() {
                    Some(cell) => self.add_cell(cell.clone(), span),
                    None => {}
                }
            }
        }
    }

    fn derive_char(self, span: Span, c: char) -> Cell {
        let next_cell = &mut cell![;span];

        match Symbol::from_char(c) {
            Some(symbol) => {
                if let Some(symbols) = self.grammar.clone().derive_single(symbol) {
                    for symbol in symbols {
                        next_cell.push_nodes(Node::new(
                            symbol,
                            span,
                            vec![]
                        ))
                    }
                }
            }
            None => {}
        }

        next_cell.clone()
    }

    fn derive(self, span: Span, base: &Cell, suffix: &Cell) -> Cell {
        let next_cell = &mut cell![;span];

        for cur in &base.0 {
            for suffix in &suffix.0 {
                if let Some(symbols) = self.grammar.clone().follow(cur.clone().kind()) {
                    if symbols.iter().any(|&sym| sym.eq(&suffix.clone().kind())) {
                        // println!("pre derive");
                        if let Some(symbols) = self.grammar.clone().derive(cur.clone().kind(), suffix.clone().kind()) {
                            // println!("derive result:{:?}", symbols);
                            for symbol in symbols {
                                next_cell.push_nodes(Node::new(
                                    symbol,
                                    span,
                                    vec![cur.clone(), suffix.clone()]
                                ))
                            }
                        }
                    }
                }
            }
        }

        if next_cell.clone().len() > 0 {
            next_cell.clone()
        } else {
            base.clone()
        }
    }

    fn get_cell(&self, span: Span) -> Option<&Cell> {
        self.slices.get(&span)
    }

    fn spans_from_len(self, len: usize) -> Vec<Span> {
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

    fn src_len(self) -> usize {
        self.chars.len()
    }
}
