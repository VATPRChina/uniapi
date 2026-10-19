use std::fmt::{self, Display};

use super::{ConstructedLeg, Ident, IdentWithCandidate, LexerToken, SolvedIdent};
use crate::modules::flight::dto::FlightRouteFix;
use crate::modules::flight::flight_plan::RouteSegment;
use crate::modules::navdata::models::{AnyFix, LegKind, PreferredRoute, ResolvedLeg};

/// Borrowed snapshots of the shared parser pipeline, in execution order.
pub enum RouteParseStep<'a, 's> {
    Lexed(&'a [LexerToken<'s>]),
    Parsed(&'a [Ident<'s>]),
    Candidates(&'a [IdentWithCandidate<'s>]),
    Solved(&'a [SolvedIdent<'s>]),
    Constructed(&'a [ConstructedLeg]),
    Expanded(&'a [ResolvedLeg]),
    PreferredRoutes(&'a [(RouteSegment<'a>, Option<PreferredRoute>)]),
}

impl Display for RouteParseStep<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexed(tokens) => {
                writeln!(f, "=== 1. Lexer: {} tokens ===", tokens.len())?;
                for (index, token) in tokens.iter().enumerate() {
                    writeln!(f, "  [{index}] {:?}", token)?;
                }
            }
            Self::Parsed(idents) => {
                writeln!(f, "=== 2. Parser: {} entries ===", idents.len())?;
                for (index, ident) in idents.iter().enumerate() {
                    writeln!(f, "  [{index}] {:?}", ident)?;
                }
            }
            Self::Candidates(idents) => {
                writeln!(f, "=== 3. Candidate resolver: {} entries ===", idents.len())?;
                for (index, ident) in idents.iter().enumerate() {
                    writeln!(f, "  [{index}] {:#?}", ident)?;
                }
            }
            Self::Solved(idents) => {
                writeln!(f, "=== 4. Solver: {} entries ===", idents.len())?;
                writeln!(
                    f,
                    "  Distances are in NM; positions are latitude/longitude; indices are zero-based."
                )?;
                for (index, ident) in idents.iter().enumerate() {
                    writeln!(
                        f,
                        "  [{index}] {}: {} surviving states",
                        ident.ident.identifier(),
                        ident.candidates.len()
                    )?;
                    for (candidate_index, candidate) in ident.candidates.iter().enumerate() {
                        writeln!(f, "      [{index}:{candidate_index}]")?;
                        for line in format!("{candidate:#?}").lines() {
                            writeln!(f, "        {line}")?;
                        }
                        if let Some(previous) = index.checked_sub(1) {
                            writeln!(
                                f,
                                "        predecessor=[{previous}:{}]",
                                candidate.last_candidate_idx
                            )?;
                        } else {
                            writeln!(f, "        predecessor=none (departure)")?;
                        }
                    }
                }
            }
            Self::Constructed(legs) => {
                write_legs(f, "5. Constructor", legs.iter().map(|entry| &entry.leg))?
            }
            Self::Expanded(legs) => write_legs(f, "6. Expander", legs.iter())?,
            Self::PreferredRoutes(matches) => {
                writeln!(f, "=== 7. Preferred routes: {} segments ===", matches.len())?;
                for (index, (segment, preferred)) in matches.iter().enumerate() {
                    writeln!(
                        f,
                        "  Segment {}: legs [{}..{}), {}",
                        index + 1,
                        segment.start_index,
                        segment.end_index(),
                        preferred
                            .as_ref()
                            .map_or("no matching preferred route", |route| route.name.as_str())
                    )?;
                    for (offset, leg) in segment.legs.iter().enumerate() {
                        write_leg(f, segment.start_index + offset, leg)?;
                    }
                }
            }
        }
        writeln!(f)
    }
}

fn write_legs<'a>(
    f: &mut fmt::Formatter<'_>,
    stage: &str,
    legs: impl ExactSizeIterator<Item = &'a ResolvedLeg>,
) -> fmt::Result {
    writeln!(f, "=== {stage}: {} legs ===", legs.len())?;
    for (index, leg) in legs.enumerate() {
        write_leg(f, index, leg)?;
    }
    Ok(())
}

fn write_leg(f: &mut fmt::Formatter<'_>, index: usize, leg: &ResolvedLeg) -> fmt::Result {
    let procedure_tag = match leg.kind {
        LegKind::Sid => " [SID]",
        LegKind::Star => " [STAR]",
        _ => "",
    };
    writeln!(
        f,
        "  [{index}] {} --{}{}{}--> {}",
        fix_text(&leg.from),
        leg.identifier.as_deref().unwrap_or("DCT"),
        if leg.is_unknown { " [UNK]" } else { "" },
        procedure_tag,
        fix_text(&leg.to)
    )?;
    writeln!(
        f,
        "      kind={:?}, direction_restriction={:?}",
        leg.kind, leg.direction_restriction
    )?;
    Ok(())
}

fn fix_text(fix: &AnyFix) -> String {
    let fix = FlightRouteFix::from(fix);
    match (fix.latitude, fix.longitude) {
        (Some(lat), Some(lon)) => format!("{} ({lat:.6}, {lon:.6})", fix.identifier),
        _ => format!("{} [UNK]", fix.identifier),
    }
}
