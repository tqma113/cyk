use super::Symbol;

use std::cmp::{Ord, Ordering};
use std::fmt;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span(usize, usize);

impl Span {
    pub fn new(start: usize, len: usize) -> Self {
        Span(start, len)
    }

    pub fn as_str(self) -> &'static str {
        let string: &'static str =
            unsafe { &*(format!("{},{}", self.0, self.1).as_str() as *const str) };
        string
    }

    pub fn start(self) -> usize {
        self.0
    }

    pub fn end(self) -> usize {
        self.0 + self.1
    }

    pub fn len(self) -> usize {
        self.1
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_str(), f)
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len().cmp(&other.len())
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub enum NodeChildren {
    None,
    Single(Box<Node>),
    Double(Box<Node>, Box<Node>),
}

#[derive(Clone)]
pub struct Node {
    kind: Symbol,
    span: Span,
    children: NodeChildren,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind) && self.span.eq(&other.span)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self.clone().children() {
            NodeChildren::Double(left, right) => {
                format!("{}{}", left, right)
            }
            NodeChildren::Single(child) => format!("{}", child.as_ref()),
            NodeChildren::None => format!("{}", self.clone().kind()),
        })
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .field("value", &self.to_string())
            .finish()
    }
}

impl Node {
    pub fn new(kind: Symbol, span: Span, children: NodeChildren) -> Self {
        Node {
            kind,
            span,
            children,
        }
    }

    pub fn kind(self) -> Symbol {
        self.kind
    }

    pub fn span(self) -> Span {
        self.span
    }

    pub fn children(self) -> NodeChildren {
        self.children
    }
}

#[derive(Debug, Clone)]
pub struct Cell(pub Vec<Node>, Span);

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.clone()
            .span()
            .unwrap()
            .cmp(&other.clone().span().unwrap())
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        if self.clone().len() != other.clone().len() {
            return false;
        }

        for i in 0..self.clone().len() {
            if !self
                .clone()
                .nth(i)
                .unwrap()
                .eq(&other.clone().nth(i).unwrap())
            {
                return false;
            }
        }

        true
    }
}

impl Eq for Cell {}

#[macro_export]
macro_rules! cell {
    (;$span:expr) => (
        Cell::new(Vec::new(), $span)
    );
    ($elem:expr; $n:expr; $span:expr) => (
        Cell::new(vec::from_elem($elem, $n))
    );
    ($($x:expr),+ $(,)?; $span:expr) => (
        <Cell>::new([$($x),+].to_vec(), $span)
    );
}

impl Cell {
    pub fn new(nodes: Vec<Node>, span: Span) -> Self {
        Cell(nodes, span)
    }

    pub fn nth(self, n: usize) -> Option<Node> {
        match self.0.get(n) {
            Some(fg) => Some(fg.clone()),
            None => None,
        }
    }

    pub fn span(self) -> Option<Span> {
        match self.0.last() {
            Some(node) => Some(node.span),
            None => None,
        }
    }

    pub fn len(self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn append(&mut self, mut another: Cell) {
        self.0.append(another.0.as_mut())
    }

    pub fn append_nodes(&mut self, mut nodes: Vec<Node>) {
        self.0.append(&mut nodes)
    }

    pub fn push_nodes(&mut self, node: Node) {
        self.0.push(node)
    }

    pub fn has(&self, symbol: Symbol) -> Option<Node> {
        match self.0.iter().find(|node| node.kind.eq(&symbol)) {
            Some(node) => Some(node.clone()),
            None => None
        }
    }
}
