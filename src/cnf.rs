use super::symbol::*;
use super::Grammar;

pub use std::collections::HashSet;

#[macro_export]
macro_rules! cnf_grammar {
    (
        Start($start:literal);
        NonTerminals[$($non_terminal:literal),+ $(,)?];
        Terminals[$($terminal:literal),+ $(,)?];
        Rules[$($left:literal => [$([$first:literal,$second:literal]),+ $(,)?]),+ $(,)?];
        TerminalRules[$($t_left:literal => [$($t_right:literal),+ $(,)?]),+ $(,)?]
    ) => {
        {
            let start_terminal = $crate::Symbol::intern($start);

            let mut non_terminals: $crate::HashSet<$crate::Symbol> = $crate::HashSet::new();
            $(
                non_terminals.insert($crate::Symbol::intern($non_terminal));
            )*

            let mut terminals: $crate::HashSet<$crate::Symbol> = $crate::HashSet::new();
            $(
                let symbol = $crate::Symbol::intern($terminal);
                assert!(
                    !non_terminals.contains(&symbol),
                    format!("Non-terminal:{} has already exist in terminal set.", symbol)
                );
                terminals.insert(symbol);
            )*

            let mut rules = $crate::Rules::new();
            $(
                let left = $crate::Symbol::intern($left);
                assert!(
                    non_terminals.contains(&left),
                    format!("The rule's left part: {} is in non-terminals", left)
                );
                let mut right: $crate::HashSet<$crate::RuleRight> = $crate::HashSet::new();
                $(
                    let first = $crate::Symbol::intern($first);
                    let second = $crate::Symbol::intern($second);
                    assert!(
                        terminals.contains(&first) || non_terminals.contains(&first),
                        format!("The rule's first part: {} is in non-terminal or terminal set", first)
                    );
                    assert!(
                        terminals.contains(&second) || non_terminals.contains(&second),
                        format!("The rule's second part: {} is in non-terminal or terminal set", second)
                    );
                    right.insert($crate::RuleRight::new(first, second));
                )*
                rules.insert(left, right);
            )*

            let mut terminal_rules = $crate::TerminalRules::new();
            $(
                let left = $crate::Symbol::intern($t_left);
                assert!(
                    non_terminals.contains(&left),
                    format!("The rule's left part: {} is in non-terminal set", left)
                );
                let mut right: $crate::HashSet<$crate::Symbol> = $crate::HashSet::new();
                $(
                    let symbol = $crate::Symbol::intern($t_right);
                    assert!(
                        terminals.contains(&symbol),
                        format!("The rule's left part: {} is in non-terminal set", left)
                    );
                    right.insert(symbol);
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct RuleRight(Symbol, Symbol);

impl RuleRight {
    pub fn new(left: Symbol, right: Symbol) -> Self {
        RuleRight(left, right)
    }
}

#[derive(Debug, Clone)]
pub struct Rule(Symbol, HashSet<RuleRight>);

impl Rule {
    pub fn first(&self) -> Option<HashSet<Symbol>> {
        let mut result: HashSet<Symbol> = HashSet::new();

        for branch in &self.1 {
            result.insert(branch.0);
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }

    pub fn follow(&self, symbol: Symbol) -> Option<Vec<Symbol>> {
        let mut result: Vec<Symbol> = vec![];

        for branch in &self.1 {
            if branch.0 == symbol {
                result.push(branch.1)
            }
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }

    pub fn start(&self) -> Symbol {
        self.0
    }

    pub fn start_with(&self, symbol: Symbol) -> bool {
        self.0 == symbol
    }

    pub fn derive(&self, base: Symbol, suffix: Symbol) -> Option<Symbol> {
        match self.follow(base) {
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

impl Default for Rules {
    fn default() -> Self {
        Self::new()
    }
}

impl Rules {
    pub fn new() -> Self {
        let rules: Vec<Rule> = vec![];
        Rules(rules)
    }

    pub fn insert(&mut self, left: Symbol, right: HashSet<RuleRight>) {
        self.0.push(Rule(left, right))
    }

    pub fn first(&self, symbol: Symbol) -> Option<HashSet<Symbol>> {
        let mut result: HashSet<Symbol> = HashSet::new();

        for rule in &self.0 {
            if rule.start_with(symbol) {
                if let Some(symbols) = rule.first() {
                    result.extend(symbols)
                }
            }
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }

    pub fn follow(&self, symbol: Symbol) -> Option<HashSet<Symbol>> {
        let mut result: HashSet<Symbol> = HashSet::new();

        for rule in &self.0 {
            if let Some(symbols) = rule.follow(symbol) {
                result.extend(symbols)
            }
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }

    pub fn derive(&self, base: Symbol, suffix: Symbol) -> Option<HashSet<Symbol>> {
        let mut result: HashSet<Symbol> = HashSet::new();

        for rule in &self.0 {
            if let Some(symbol) = rule.derive(base, suffix) {
                result.insert(symbol);
            }
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerminalRule(Symbol, HashSet<Symbol>);

impl TerminalRule {
    pub fn start(&self) -> Symbol {
        self.0
    }

    fn derive(&self, base: Symbol) -> Option<Symbol> {
        for symbol in &self.1 {
            if symbol.eq(&base) {
                return Some(self.0);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct TerminalRules(Vec<TerminalRule>);

impl Default for TerminalRules {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalRules {
    pub fn new() -> Self {
        let rules: Vec<TerminalRule> = vec![];
        TerminalRules(rules)
    }

    pub fn insert(&mut self, left: Symbol, right: HashSet<Symbol>) {
        self.0.push(TerminalRule(left, right))
    }

    fn derive(&self, base: Symbol) -> Option<HashSet<Symbol>> {
        let mut result: HashSet<Symbol> = HashSet::new();

        for rule in &self.0 {
            if let Some(symbol) = rule.derive(base) {
                result.insert(symbol);
            }
        }

        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CNF {
    start: Symbol,
    terminals: HashSet<Symbol>,
    non_terminals: HashSet<Symbol>,
    rules: Rules,
    terminal_rules: TerminalRules,
}

impl CNF {
    pub fn new(
        start: Symbol,
        terminals: HashSet<Symbol>,
        non_terminals: HashSet<Symbol>,
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
    fn start_symbol(&self) -> Symbol {
        self.start
    }

    fn exist(&self, symbol: Symbol) -> bool {
        self.non_terminals.contains(&symbol) || self.terminals.contains(&symbol)
    }

    fn first(&self, symbol: Symbol) -> Option<HashSet<Symbol>> {
        self.rules.first(symbol)
    }

    fn follow(&self, symbol: Symbol) -> Option<HashSet<Symbol>> {
        self.rules.follow(symbol)
    }

    fn derive(&self, base: Symbol, suffix: Symbol) -> Option<HashSet<Symbol>> {
        self.rules.derive(base, suffix)
    }

    fn derive_single(&self, base: Symbol) -> Option<HashSet<Symbol>> {
        self.terminal_rules.derive(base)
    }

    fn is_terminal(&self, input: Symbol) -> bool {
        self.terminals.contains(&input)
    }

    fn is_non_terminal(&self, input: Symbol) -> bool {
        self.non_terminals.contains(&input)
    }
}
