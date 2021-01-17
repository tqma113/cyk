use super::symbol::*;
use super::Grammar;

#[macro_export]
macro_rules! cnf_grammar {
    (
        Start($start:literal);NonTerminals[$($non_terminal:literal),+ $(,)?];
        Terminals[$($terminal:literal),+ $(,)?];
        Rules[$($left:literal => [$([$first:literal,$second:literal]),+ $(,)?]),+ $(,)?];
        TerminalRules[$($t_left:literal => [$($t_right:literal),+ $(,)?]),+ $(,)?]
    ) => {
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
                let mut right: Vec<$crate::RuleRight> = vec![];
                $(
                    right.push($crate::RuleRight::new(
                        $crate::Symbol::intern($first),
                        $crate::Symbol::intern($second)
                    ));
                )*
                rules.insert(left, right);
            )*

            let mut terminal_rules = $crate::TerminalRules::new();
            $(
                let left = $crate::Symbol::intern($t_left);
                let mut right: Vec<$crate::Symbol> = vec![];
                $(
                    right.push($crate::Symbol::intern($t_right));
                )*
                terminal_rules.insert(left, right);
            )*

            $crate::CNF::new(
                start_terminal,
                non_terminals,
                terminals,
                rules,
                terminal_rules
            )
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct RuleRight(Symbol, Symbol);

impl RuleRight {
    pub fn new(left: Symbol, right: Symbol) -> Self {
        RuleRight(left, right)
    }
}

#[derive(Debug, Clone)]
pub struct Rule(Symbol, Vec<RuleRight>);

impl Rule {
    pub fn first(self) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for branch in self.1 {
            result.push(branch.0)
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
            if branch.0 == symbol {
                result.push(branch.1)
            }
        }

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    pub fn start(self) -> Symbol {
        self.0
    }

    pub fn start_with(self, symbol: Symbol) -> bool {
        self.0 == symbol
    }

    pub fn derive(&self, base: Symbol, suffix: Symbol) -> Option<Symbol> {
        match self.clone().follow(base) {
            Some(symbols) => {
                if symbols.contains(&suffix) {
                    Some(self.0)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rules(Vec<Rule>);

impl Rules {
    pub fn new() -> Self {
        let rules: Vec<Rule> = vec![];
        Rules(rules)
    }

    pub fn insert(&mut self, left: Symbol, right: Vec<RuleRight>) {
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
}

#[derive(Debug, Clone)]
pub struct TerminalRule(Symbol, Vec<Symbol>);

impl TerminalRule {
    pub fn start(self) -> Symbol {
        self.0
    }

    fn derive(self, base: Symbol) -> Option<Symbol> {
        for symbol in self.1 {
            if symbol.eq(&base) {
                return Some(self.0);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct TerminalRules(Vec<TerminalRule>);

impl TerminalRules {
    pub fn new() -> Self {
        let rules: Vec<TerminalRule> = vec![];
        TerminalRules(rules)
    }

    pub fn insert(&mut self, left: Symbol, right: Vec<Symbol>) {
        self.0.push(TerminalRule(left, right))
    }

    fn derive(self, base: Symbol) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for rule in self.0 {
            if let Some(symbol) = rule.derive(base) {
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
    terminal_rules: TerminalRules,
}

impl CNF {
    pub fn new(
        start: Symbol,
        terminals: Vec<Symbol>,
        non_terminals: Vec<Symbol>,
        rules: Rules,
        terminal_rules: TerminalRules,
    ) -> Self {
        CNF {
            start,
            terminals,
            non_terminals,
            rules,
            terminal_rules,
        }
    }
}

impl Grammar for CNF {
    fn start_symbol(self) -> Symbol {
        self.start
    }

    fn exist(self, symbol: Symbol) -> bool {
        self.non_terminals.contains(&symbol) || self.terminals.contains(&symbol)
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
        self.terminal_rules.derive(base)
    }

    fn is_terminal(self, input: Symbol) -> bool {
        self.terminals.contains(&input)
    }

    fn is_non_terminal(self, input: Symbol) -> bool {
        self.non_terminals.contains(&input)
    }
}
