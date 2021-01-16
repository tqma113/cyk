use std::fmt;

use rustc_hash::{FxHashMap};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(usize);

impl Symbol {
    const fn new(n: u32) -> Self {
        Symbol(n as usize)
    }

    /// Maps a string to its interned representation.
    pub fn intern(string: &str) -> Self {
        with_interner(|interner| interner.intern(string))
    }

    /// Convert to a `SymbolStr`. This is a slowish operation because it
    /// requires locking the symbol interner.
    pub fn as_str(self) -> &'static str {
        with_interner(|interner| unsafe {
            std::mem::transmute::<&str, &str>(interner.get(self))
        })
    }

    pub fn as_u32(self) -> u32 {
        self.0 as u32
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_str(), f)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.as_str(), f)
    }
}

#[derive(Default)]
pub struct Interner {
    names: FxHashMap<&'static str, Symbol>,
    strings: Vec<&'static str>,
}

impl Interner {
    fn prefill(init: &[&'static str]) -> Self {
        Interner {
            strings: init.into(),
            names: init.iter().copied().zip((0..).map(Symbol::new)).collect(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn intern(&mut self, string: &str) -> Symbol {
        if let Some(&name) = self.names.get(string) {
            return name;
        }

        let name = Symbol::new(self.strings.len() as u32);

        // It is safe to extend the arena allocation to `'static` because we only access
        // these while the arena is still alive.
        let string: &'static str = unsafe { &*(string as *const str) };
        self.strings.push(string);
        self.names.insert(string, name);
        name
    }

    // Get the symbol as a string. `Symbol::as_str()` should be used in
    // preference to this function.
    pub fn get(&self, symbol: Symbol) -> &str {
        self.strings[symbol.0]
    }
}

pub struct SessionGlobals {
    symbol_interner: std::cell::RefCell<Interner>,
}

impl SessionGlobals {
    pub fn new() -> SessionGlobals {
        SessionGlobals {
            symbol_interner: std::cell::RefCell::new(Interner::default())
        }
    }
}

scoped_tls::scoped_thread_local!(pub static SESSION_GLOBALS: SessionGlobals);

fn with_interner<T, F: FnOnce(&mut Interner) -> T>(f: F) -> T {
    SESSION_GLOBALS.with(|session_globals| f(&mut *session_globals.symbol_interner.borrow_mut()))
}
pub fn with_session_globals(f: impl FnOnce()) {
    let session_globals = SessionGlobals::new();
    SESSION_GLOBALS.set(&session_globals, f);
}
