use std::collections::HashSet;

use arrayvec::ArrayString;

use super::AnyFix;

#[derive(Debug, Clone, PartialEq)]
pub struct Airway {
    pub identifier: ArrayString<7>,
    pub fix_identifiers: HashSet<ArrayString<7>>,
}

impl Airway {
    pub fn contains_fix(&self, fix: &AnyFix) -> bool {
        fix.identifier().is_some_and(|identifier| {
            self.fix_identifiers
                .iter()
                .any(|candidate| candidate.as_str() == identifier)
        })
    }
}
