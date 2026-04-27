use std::collections::{HashMap, HashSet, VecDeque};

use sqlx::PgPool;

use crate::{
    flight_plan::{
        AirwayDirection, AirwayLeg, DirectLeg, Fix, Leg, RouteToken,
        lexer::{LexerError, lex_route},
    },
    repository::airway,
};

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("lexer error: {0}")]
    Lexer(#[from] LexerError),
    #[error("no initial fix in route")]
    MissingInitialFix,
    #[error("unexpected token {0}")]
    UnexpectedToken(String),
}

pub async fn parse_route(db: &PgPool, route: &str) -> Result<Vec<Leg>, ParserError> {
    let tokens = lex_route(db, route).await?;
    RouteParser::new(db, tokens).parse().await
}

struct RouteParser<'a> {
    db: &'a PgPool,
    tokens: Vec<RouteToken>,
    legs: Vec<Leg>,
    last_fix_override: Option<Fix>,
}

impl<'a> RouteParser<'a> {
    fn new(db: &'a PgPool, tokens: Vec<RouteToken>) -> Self {
        Self {
            db,
            tokens,
            legs: Vec::new(),
            last_fix_override: None,
        }
    }

    async fn parse(mut self) -> Result<Vec<Leg>, ParserError> {
        for index in 0..self.tokens.len() {
            let token = self.tokens[index].clone();
            if self.last_fix_override.is_none() && self.legs.is_empty() {
                let RouteToken::Fix { fix, .. } = &token else {
                    return Err(ParserError::MissingInitialFix);
                };
                self.last_fix_override = Some(fix.clone());
            } else if !self.legs.is_empty() {
                self.last_fix_override = None;
            }

            match token {
                RouteToken::SidLeg { .. }
                | RouteToken::StarLeg { .. }
                | RouteToken::SpeedAndAltitude { .. }
                | RouteToken::DirectLeg { .. } => {}
                RouteToken::AirwayLeg { value } => self.handle_airway(index, &value).await?,
                RouteToken::Fix { value, fix } => self.handle_waypoint(&value, fix)?,
                RouteToken::Unknown { value } => return Err(ParserError::UnexpectedToken(value)),
            }
        }

        Ok(self.legs)
    }

    async fn handle_airway(&mut self, index: usize, ident: &str) -> Result<(), ParserError> {
        let leg_lookup = airway_leg_lookup(airway::legs(self.db, ident).await?);
        let Some(from_fix) = index.checked_sub(1).and_then(|i| self.tokens.get(i)) else {
            return Ok(());
        };
        let Some(to_fix) = self.tokens.get(index + 1) else {
            return Ok(());
        };
        let path = bfs_path(from_fix.value(), to_fix.value(), &leg_lookup);
        self.legs.extend(path.into_iter().map(Leg::Airway));
        Ok(())
    }

    fn handle_waypoint(&mut self, _value: &str, fix: Fix) -> Result<(), ParserError> {
        let last_fix = self.last_fix()?;
        if last_fix.latitude == fix.latitude && last_fix.longitude == fix.longitude {
            return Ok(());
        }
        self.legs.push(Leg::Direct(DirectLeg {
            from: last_fix,
            to: fix,
        }));
        Ok(())
    }

    fn last_fix(&self) -> Result<Fix, ParserError> {
        self.last_fix_override
            .clone()
            .or_else(|| self.legs.last().map(|leg| leg.to().clone()))
            .ok_or(ParserError::MissingInitialFix)
    }
}

fn airway_leg_lookup(legs: Vec<AirwayLeg>) -> HashMap<String, HashMap<String, AirwayLeg>> {
    let mut lookup: HashMap<String, HashMap<String, AirwayLeg>> = HashMap::new();
    for leg in legs {
        let Some(from_ident) = leg.from.identifier.clone() else {
            continue;
        };
        let Some(to_ident) = leg.to.identifier.clone() else {
            continue;
        };

        lookup
            .entry(from_ident.clone())
            .or_default()
            .insert(to_ident.clone(), leg.clone());

        lookup.entry(to_ident).or_default().insert(
            from_ident,
            AirwayLeg {
                from: leg.to,
                to: leg.from,
                identifier: leg.identifier,
                direction: match leg.direction {
                    AirwayDirection::Forward => AirwayDirection::Backward,
                    AirwayDirection::Backward => AirwayDirection::Forward,
                    AirwayDirection::Both => AirwayDirection::Both,
                },
            },
        );
    }
    lookup
}

fn bfs_path(
    from: &str,
    to: &str,
    legs: &HashMap<String, HashMap<String, AirwayLeg>>,
) -> Vec<AirwayLeg> {
    let mut queue = VecDeque::from([from.to_owned()]);
    let mut visited = HashSet::from([from.to_owned()]);
    let mut parent: HashMap<String, String> = HashMap::new();

    while let Some(current) = queue.pop_front() {
        if current == to {
            break;
        }

        let Some(next_legs) = legs.get(&current) else {
            continue;
        };
        for next in next_legs.keys() {
            if visited.insert(next.clone()) {
                parent.insert(next.clone(), current.clone());
                queue.push_back(next.clone());
            }
        }
    }

    let mut path = Vec::new();
    let mut current = to.to_owned();
    while current != from {
        let Some(previous) = parent.get(&current) else {
            return Vec::new();
        };
        let Some(leg) = legs.get(previous).and_then(|next| next.get(&current)) else {
            return Vec::new();
        };
        path.push(leg.clone());
        current = previous.clone();
    }
    path.reverse();
    path
}
