pub use super::symbol::*;

#[macro_export]
macro_rules! cnf {
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
                        branch.len() <= 2,
                        "The maximum length of a right-hand side in a rule is 2 in CNF"
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
    pub fn first(self) -> Vec<Symbol> {
        let mut result: Vec<Symbol> = vec![];

        for branch in self.1 {
            result.push(*branch.first().unwrap())
        }

        result
    }

    pub fn start_with(self, symbol: Symbol) -> bool {
        self.0 == symbol
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

    pub fn first(self, symbol: Symbol) -> Vec<Symbol> {
        let mut result: Vec<Symbol> = vec![];

        for rule in self.0 {
            if rule.clone().start_with(symbol) {
                result.append(&mut rule.first())
            }
        }

        result
    }
}

#[derive(Debug)]
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
