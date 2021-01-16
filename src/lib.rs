mod symbol;

pub use symbol::*;

#[macro_export]
macro_rules! cyk {
    (Start($start:literal);NonTerminal[$($non_terminal:literal),+ $(,)?];Terminal[$($terminal:literal),+ $(,)?];Rules[$($left:literal => [$([$($right:literal),+ $(,)?]),+ $(,)?]),+ $(,)?]) => {
        {
            let start_terminal = $start;
            let non_terminals = [$($non_terminal),+];
            let terminals = [$($terminal),+];
            let rules = [$(($left, [$([$($right),+]),+])),+];

            let start_terminal = $crate::Symbol::intern(start_terminal);

            start_terminal
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
struct Rules(Vec<Rule>);

impl Rules {
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

struct CYKParser {
    start: Symbol,
    terminals: Vec<Symbol>,
    non_terminals: Vec<Symbol>,
    rules: Rules
}

