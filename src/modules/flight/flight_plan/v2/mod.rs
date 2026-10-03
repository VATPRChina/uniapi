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
//! Route legs
//!         ▼ Expander ◄── NavData
//! Published leg segments
//! ```

#![allow(unused)]

mod constructor;
mod expander;
mod lexer;
mod parser;
mod resolver;
mod solver;

pub use constructor::Constructor;
pub use expander::Expander;
pub use lexer::{CruisingLevel, Lexer, LexerToken, LexerTokenAmend, LexerTokenValue, Speed};
pub use parser::{Ident, IdentAmend, Parser};
pub use resolver::{
    CandidateResolver, FixCandidate, IdentCandidate, IdentWithCandidate, LegCandidate,
};
pub use solver::{CandidateWithState, SolvedIdent, Solver};

use crate::modules::navdata::{
    models::ResolvedLeg,
    service::{InvalidNavdataError, NavdataService},
};

#[derive(Debug, thiserror::Error)]
pub enum ParseRouteError {
    #[error("invalid flight route: {0}")]
    InvalidRoute(String),
    #[error("failed to load route navigation data: {0}")]
    Navdata(#[from] InvalidNavdataError),
}

/// Parse and expand complete route text using the shared API/CLI pipeline.
pub async fn parse_route(
    navdata: &NavdataService,
    route: &str,
) -> Result<Vec<ResolvedLeg>, ParseRouteError> {
    let parsed: Vec<_> = Parser::new(Lexer::new(route).parse_all().collect())
        .parse()
        .collect();
    if parsed.is_empty() {
        return Err(ParseRouteError::InvalidRoute("route is empty".to_owned()));
    }
    if let Some(ident) = parsed.iter().find(|ident| !ident.errors.is_empty()) {
        return Err(ParseRouteError::InvalidRoute(format!(
            "invalid route entry {:?}: {:?}",
            ident.identifier(),
            ident.errors
        )));
    }
    let candidates = CandidateResolver::new(parsed)
        .resolve_candidates(navdata)
        .await?
        .collect();
    let solved = Solver::new(candidates, navdata)
        .solve()
        .into_iter()
        .collect();
    let constructed = Constructor::new(solved).construct();
    if constructed.is_empty() {
        return Err(ParseRouteError::InvalidRoute(
            "no complete route could be constructed".to_owned(),
        ));
    }
    Ok(Expander::new(constructed).expand(navdata).await?)
}
