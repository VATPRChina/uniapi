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
//! ```

#![allow(unused)]

mod candidate_resolver;
mod constraint_solver;
mod lexer;
mod parser;

pub use lexer::Lexer;
