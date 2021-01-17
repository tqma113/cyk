use super::symbol::*;
use super::Grammar;

#[macro_export]
macro_rules! cnf_grammar {
    (Start($start:literal);NonTerminal[$($non_terminal:literal),+ $(,)?];Terminal[$($terminal:literal),+ $(,)?];Rules[$($left:literal => [$([$($right:literal),+ $(,)?]),+ $(,)?]),+ $(,)?]) => {
        {
            let start_terminal = $crate::Symbol::intern($start);

            let mut non_terminals: Vec<$crate::Symbol> = vec![];
            $(
                non_terminals.push($crate::Symbol::intern($non_terminal));
            )*

            let mut terminals: Vec<$crate::Symbol> = vec![];
            $(
                terminals.push($crate::Symbol::intern($terminal));
            )*

            let mut rules = $crate::Rules::new();
            $(
                let left = $crate::Symbol::intern($left);
                assert!(
                    non_terminals.iter().any(|&symbol| symbol == left),
                    format!("The rule's left part: {} is in non-terminals", left)
                );
                let mut right: Vec<Vec<$crate::Symbol>> = vec![];
                $(
                    let mut branch: Vec<$crate::Symbol> = vec![];
                    $(
                        let symbol = $crate::Symbol::intern($right);
                        assert!(
                            non_terminals.iter().any(|&sym| sym == symbol) || terminals.iter().any(|&sym| sym == symbol),
                            format!("The rule's right part: {} is in terminals or non-terminals", symbol)
                        );
                        branch.push(symbol);
                    )*
                    assert!(
                        branch.len() == 2 || branch.len() == 1,
                        "The length of a right-hand side in a rule should be 1 or 2 in CNF"
                    );
                    right.push(branch);
                )*
                rules.insert_by_symbol(left, right);
            )*



            $crate::CNF::new(start_terminal, non_terminals, terminals, rules)
        }
    };
}

#[derive(Debug, Clone)]
struct Rule(Symbol, Vec<Vec<Symbol>>);

impl Rule {
    pub fn first(self) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for branch in self.1 {
            result.push(*branch.first().unwrap())
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    pub fn follow(self, symbol: Symbol) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for branch in self.1 {
            if branch.len() == 2 && *branch.first().unwrap() == symbol {
                result.push(*branch.get(1).unwrap())
            }
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    pub fn start_with(self, symbol: Symbol) -> bool {
        self.0 == symbol
    }

    pub fn derive(&self, base: Symbol, suffix: Symbol) -> Option<Symbol> {
        match self.clone().follow(base) {
            Some(symbols) => {
                if symbols.iter().any(|sym| sym.eq(&suffix)) {
                    Some(self.0)
                } else {
                    None
                }
            }
            None => None
        }
    }

    fn derive_single(self, base: Symbol) -> Option<Symbol> {
        for symbols in self.1 {
            if symbols.len() == 1 && symbols.first().unwrap().eq(&base) {
                return Some(self.0)
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Rules(Vec<Rule>);

impl Rules {
    pub fn new() -> Self {
        let rules: Vec<Rule> = vec![];
        Rules(rules)
    }
    pub fn insert(&mut self, left: &str, right: Vec<Vec<&str>>) {
        let left_symbol = Symbol::intern(left);
        let mut right_arr: Vec<Vec<Symbol>> = vec![vec![]];

        for branch in right {
            let mut branch_arr: Vec<Symbol> = vec![];
            for item in branch {
                branch_arr.push(Symbol::intern(item));
            }
            right_arr.push(branch_arr);
        }

        self.insert_by_symbol(left_symbol, right_arr)
    }

    pub fn insert_by_symbol(&mut self, left: Symbol, right: Vec<Vec<Symbol>>) {
        self.0.push(Rule(left, right))
    }

    pub fn first(self, symbol: Symbol) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for rule in self.0 {
            if rule.clone().start_with(symbol) {
                if let Some(mut symbols) = rule.first() {
                    result.append(&mut symbols)
                }
            }
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    pub fn follow(self, symbol: Symbol) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for rule in self.0 {
            if let Some(mut symbols) = rule.follow(symbol) {
                result.append(&mut symbols)
            }
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }


    pub fn derive(self, base: Symbol, suffix: Symbol) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for rule in self.0 {
            if let Some(symbol) = rule.derive(base, suffix) {
                result.push(symbol)
            }
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    fn derive_single(self, base: Symbol) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for rule in self.0 {
            if let Some(symbol) = rule.derive_single(base) {
                result.push(symbol)
            }
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CNF {
    start: Symbol,
    terminals: Vec<Symbol>,
    non_terminals: Vec<Symbol>,
    rules: Rules,
}

impl CNF {
    pub fn new(
        start: Symbol,
        terminals: Vec<Symbol>,
        non_terminals: Vec<Symbol>,
        rules: Rules,
    ) -> Self {
        CNF {
            start,
            terminals,
            non_terminals,
            rules,
        }
    }
}


impl Grammar for CNF {
    fn start_symbol(self) -> Symbol {
        self.start
    }

    fn exist(self, symbol: Symbol) -> bool {
        self.non_terminals.iter().any(|&sym| sym == symbol)
            || self.terminals.iter().any(|&sym| sym == symbol)
    }

    fn first(self, symbol: Symbol) -> Option<Vec<Symbol>> {
        self.rules.first(symbol)
    }

    fn follow(self, symbol: Symbol) -> Option<Vec<Symbol>> {
        self.rules.follow(symbol)
    }

    fn derive(self, base: Symbol, suffix: Symbol) -> Option<Vec<Symbol>> {
        self.rules.derive(base, suffix)
    }

    fn derive_single(self, base: Symbol) -> Option<Vec<Symbol>> {
        self.rules.derive_single(base)
    }
}
