use arrayvec::ArrayString;

use super::{
    CandidateWithState, FixCandidate, IdentCandidate, LegCandidate, Lexer, LexerTokenValue,
    SolvedIdent,
};
use crate::modules::navdata::models::{
    Airport, AnyFix, DirectionRestriction, GeoPoint, Ndb, NdbKind, ResolvedLeg, Vhf, Waypoint,
    WaypointKind,
};

type PathEntry<'a, 's> = (&'a SolvedIdent<'s>, &'a CandidateWithState);

pub struct Constructor<'s> {
    idents: Vec<SolvedIdent<'s>>,
}

impl<'s> Constructor<'s> {
    pub fn new(idents: Vec<SolvedIdent<'s>>) -> Self {
        Self { idents }
    }

    /// Construct the first complete surviving path in solver order, following
    /// saved predecessor indices rather than selecting each entry independently.
    /// Empty or incomplete input yields no legs. Named connections retain their
    /// route identifier; this synchronous step does not expand navigation data.
    /// ResolvedLeg cannot represent the parsed speed/level or flight-rule amendments.
    pub fn construct(&self) -> Vec<ResolvedLeg> {
        self.idents
            .last()
            .into_iter()
            .flat_map(|ident| ident.candidates.iter().enumerate())
            .filter(|(_, candidate)| {
                matches!(candidate.candidate, IdentCandidate::Fix(_))
                    && candidate.distance().is_finite()
            })
            .find_map(|(index, _)| self.path(index).and_then(|path| construct_path(&path)))
            .unwrap_or_default()
    }

    fn path(&self, candidate_index: usize) -> Option<Vec<PathEntry<'_, 's>>> {
        let ident_index = self.idents.len().checked_sub(1)?;
        let candidate = self.idents[ident_index].candidates.get(candidate_index)?;
        let backwards: Vec<_> =
            std::iter::successors(Some((ident_index, candidate)), |(index, candidate)| {
                let previous = index.checked_sub(1)?;
                self.idents[previous]
                    .candidates
                    .get(candidate.last_candidate_idx)
                    .map(|candidate| (previous, candidate))
            })
            .collect();
        (backwards.len() == self.idents.len()).then(|| {
            backwards
                .into_iter()
                .rev()
                .map(|(index, candidate)| (&self.idents[index], candidate))
                .collect()
        })
    }
}

fn construct_path(path: &[PathEntry<'_, '_>]) -> Option<Vec<ResolvedLeg>> {
    if !matches!(path.first()?.1.candidate, IdentCandidate::Fix(_)) {
        return None;
    }
    let fixes: Vec<_> = path
        .iter()
        .enumerate()
        .filter_map(|(index, (ident, candidate))| match &candidate.candidate {
            IdentCandidate::Fix(fix) => Some((index, to_fix(ident.ident.identifier(), fix))),
            IdentCandidate::Leg(_) => None,
        })
        .collect();
    fixes
        .windows(2)
        .map(|pair| {
            let [(from_index, from), (to_index, to)] = pair else {
                unreachable!("windows contain two fixes")
            };
            let identifier = match &path[from_index + 1..*to_index] {
                [] => None,
                [(ident, candidate)] => match &candidate.candidate {
                    IdentCandidate::Leg(LegCandidate::Direct) => None,
                    IdentCandidate::Leg(_) => Some(ident.ident.identifier().to_owned()),
                    IdentCandidate::Fix(_) => return None,
                },
                _ => return None,
            };
            Some(ResolvedLeg {
                from: from.clone(),
                to: to.clone(),
                identifier,
                direction_restriction: DirectionRestriction::None,
            })
        })
        .collect()
}

fn to_fix(identifier: &str, candidate: &FixCandidate) -> AnyFix {
    // Reference candidates already contain the projected coordinates. Treat them
    // as points to avoid projecting twice or fitting the reference text into the
    // fixed-size identifier fields of the shared navigation models.
    if let Some((lat, lon)) = candidate.position()
        && matches!(
            Lexer::new(identifier)
                .parse_all()
                .next()
                .map(|token| token.value),
            Some(LexerTokenValue::IdentifierReference { .. })
        )
    {
        return AnyFix::GeoPoint(GeoPoint::new(lat, lon));
    }
    typed_fix(identifier, candidate).unwrap_or_else(|| AnyFix::Unknown(identifier.to_owned()))
}

fn typed_fix(identifier: &str, candidate: &FixCandidate) -> Option<AnyFix> {
    let (latitude, longitude) = candidate.position()?;
    Some(match candidate {
        FixCandidate::Airport { airport, .. } => AnyFix::Airport(Airport {
            identifier: *airport,
            latitude,
            longitude,
        }),
        FixCandidate::Geo { .. } => AnyFix::GeoPoint(GeoPoint::new(latitude, longitude)),
        FixCandidate::EnrouteWaypoint { icao_code, .. } => AnyFix::Waypoint(Waypoint {
            icao_code: ArrayString::from(icao_code.as_str()).ok()?,
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: WaypointKind::Enroute,
        }),
        // Terminal candidates retain an airport scope, but no ICAO region.
        // Shared models cannot carry that scope; leave the region empty.
        FixCandidate::TerminalWaypoint { .. } => AnyFix::Waypoint(Waypoint {
            icao_code: ArrayString::new(),
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: WaypointKind::Terminal,
        }),
        FixCandidate::EnrouteNdb { icao_code, .. } => AnyFix::Ndb(Ndb {
            icao_code: ArrayString::from(icao_code.as_str()).ok()?,
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: NdbKind::Enroute,
        }),
        FixCandidate::TerminalNdb { .. } => AnyFix::Ndb(Ndb {
            icao_code: ArrayString::new(),
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
            kind: NdbKind::Terminal,
        }),
        FixCandidate::EnrouteVor { icao_code, .. } => AnyFix::Vhf(Vhf {
            icao_code: ArrayString::from(icao_code.as_str()).ok()?,
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
        }),
        FixCandidate::TerminalVor { .. } => AnyFix::Vhf(Vhf {
            icao_code: ArrayString::new(),
            identifier: ArrayString::from(identifier).ok()?,
            latitude,
            longitude,
        }),
        FixCandidate::UnknownWaypoint => return None,
    })
}
