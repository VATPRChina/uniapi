use std::{collections::HashSet, ops::Deref};

use arrayvec::ArrayString;

#[derive(Debug, Clone, PartialEq)]
pub struct TerminalProcedure {
    pub airport: ArrayString<4>,
    pub identifier: ArrayString<8>,
    /// Nonempty set of runway codes; A/B suffixes are expanded.
    pub runway_transitions: HashSet<ArrayString<3>>,
    /// Nonempty set of named transitions, or common/runway connection fixes.
    pub enroute_transitions: HashSet<ArrayString<5>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sid {
    pub proc: TerminalProcedure,
}

impl Deref for Sid {
    type Target = TerminalProcedure;

    fn deref(&self) -> &Self::Target {
        &self.proc
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Star {
    pub proc: TerminalProcedure,
}

impl Deref for Star {
    type Target = TerminalProcedure;

    fn deref(&self) -> &Self::Target {
        &self.proc
    }
}
