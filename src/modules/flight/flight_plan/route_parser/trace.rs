use std::fmt::{self, Display};

use super::{ConstructedLeg, Ident, IdentWithCandidate, LexerToken, SolvedIdent};
use crate::modules::{
    flight::dto::FlightRouteFix,
    navdata::models::{AnyFix, LegKind, MainlandFirBoundaries, ResolvedLeg},
};

/// Borrowed snapshots of the shared parser pipeline, in execution order.
pub enum RouteParseStep<'a, 's> {
    Lexed(&'a [LexerToken<'s>]),
    Parsed(&'a [Ident<'s>]),
    Candidates(&'a [IdentWithCandidate<'s>]),
    Solved(&'a [SolvedIdent<'s>]),
    Constructed(&'a [ConstructedLeg]),
    Expanded(&'a [ResolvedLeg]),
    Mainland(&'a [ResolvedLeg], &'a MainlandFirBoundaries),
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
            Self::Mainland(legs, boundaries) => {
                let selected: Vec<_> = legs
                    .iter()
                    .enumerate()
                    .filter(|(_, leg)| boundaries.contains_either_endpoint(leg))
                    .collect();
                writeln!(
                    f,
                    "=== 7. Mainland FIR portions: {} legs (either endpoint inside) ===",
                    selected.len()
                )?;
                let mut previous = None;
                let mut part = 0;
                for (index, leg) in selected {
                    if previous != index.checked_sub(1) || previous.is_none() {
                        part += 1;
                        writeln!(f, "  Part {part}:")?;
                    }
                    write_leg(f, index, leg)?;
                    previous = Some(index);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::navdata::models::{DirectionRestriction, GeoPoint};

    #[test]
    fn mainland_trace_groups_crossings_and_retains_original_leg_indices() {
        let positions = [
            (30., 110.),
            (35., 140.),
            (36., 140.),
            (31., 111.),
            (32., 112.),
            (37., 140.),
            (38., 140.),
        ];
        let legs: Vec<_> = positions
            .windows(2)
            .enumerate()
            .map(|(index, pair)| ResolvedLeg {
                from: GeoPoint::new(pair[0].0, pair[0].1).unwrap().into(),
                to: GeoPoint::new(pair[1].0, pair[1].1).unwrap().into(),
                identifier: Some(format!("LEG{index}")),
                is_unknown: false,
                kind: if index == 0 {
                    LegKind::Sid
                } else {
                    LegKind::Airway
                },
                direction_restriction: DirectionRestriction::None,
            })
            .collect();
        let boundaries = MainlandFirBoundaries::fixture();
        let output = RouteParseStep::Mainland(&legs, &boundaries).to_string();
        assert!(output.contains("4 legs (either endpoint inside)"));
        assert!(output.contains("Part 1:"));
        assert!(output.contains("Part 2:"));
        assert!(!output.contains("Part 3:"));
        for index in [0, 2, 3, 4] {
            assert!(output.contains(&format!("[{index}]")));
            assert!(output.contains(&format!("LEG{index}")));
        }
        assert!(output.contains("[SID]"));
        for index in [1, 5] {
            assert!(!output.contains(&format!("LEG{index}")));
        }
        let empty = RouteParseStep::Mainland(&[], &boundaries).to_string();
        assert!(empty.contains("0 legs"));
        assert!(!empty.contains("Part 1:"));
    }
}
