//! # Flight plan route parser
//!
//! ## Architecture
//!
//! ```text
//! RJAA XAC/M082F350 Y28 KASMI RJTT
//!         ▼ Lexer
//! Tokens
//! [IDENT, IDENT(SPEED_LEVEL), IDENT, IDENT, IDENT]
//!         ▼ Parser
//! Syntax IR
//! [Id(RJAA), Id(XAC, M082/F350), Id(Y28), Id(KASMI), Id(RJTT)]
//!         ▼ Candidate Resolver ◄── NavData
//! Candidates
//! XAC   -> {VOR, Fix}
//! Y28   -> {Airway, Fix}
//! KASMI -> {Fix, ...}
//!         ▼ Constraint Solver ◄── NavData
//! Resolved Route
//! XAC ──Y28──> KASMI
//!  │
//!  └─ M0.82 / FL350
//!         ▼ Constructor
//! Full Route
//! ```

#![allow(unused)]

mod constructor;
mod lexer;
mod parser;
mod resolver;
mod solver;

pub use constructor::Constructor;
pub use lexer::{CruisingLevel, Lexer, LexerToken, LexerTokenAmend, LexerTokenValue, Speed};
pub use parser::{Ident, IdentAmend, Parser};
pub use resolver::{
    CandidateResolver, FixCandidate, IdentCandidate, IdentWithCandidate, LegCandidate,
};
pub use solver::{CandidateWithState, SolvedIdent, Solver};
