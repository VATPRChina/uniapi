use crate::adapter::navdata::NavdataAdapter;
use crate::flight_plan::RouteToken;
use crate::flight_plan::lexer::{LexerError, lex_route};
use crate::model::navdata::{AnyFix, DirectionRestriction, Fix, ResolvedLeg};

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("navdata error: {0}")]
    Navdata(#[from] anyhow::Error),
    #[error("lexer error: {0}")]
    Lexer(#[from] LexerError),
    #[error("no initial fix in route")]
    MissingInitialFix,
    #[error("unexpected token {0}")]
    UnexpectedToken(String),
}

pub async fn parse_route(
    navdata: &NavdataAdapter,
    route: &str,
) -> Result<Vec<ResolvedLeg>, ParserError> {
    let tokens = lex_route(navdata, route).await?;
    RouteParser::new(navdata, tokens).parse().await
}

struct RouteParser<'a> {
    navdata: &'a NavdataAdapter,
    tokens: Vec<RouteToken>,
    legs: Vec<ResolvedLeg>,
    last_fix_override: Option<AnyFix>,
}

impl<'a> RouteParser<'a> {
    fn new(navdata: &'a NavdataAdapter, tokens: Vec<RouteToken>) -> Self {
        Self {
            navdata,
            tokens,
            legs: Vec::new(),
            last_fix_override: None,
        }
    }

    async fn parse(mut self) -> Result<Vec<ResolvedLeg>, ParserError> {
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
        let Some(RouteToken::Fix {
            fix: from_fix,
            value: from_value,
        }) = index.checked_sub(1).and_then(|i| self.tokens.get(i))
        else {
            return Ok(());
        };
        let Some(RouteToken::Fix {
            fix: to_fix,
            value: to_value,
        }) = self.tokens.get(index + 1)
        else {
            return Ok(());
        };
        let airway_legs = self
            .navdata
            .list_airway_legs_between(ident, from_value, to_value)
            .await?;

        if airway_legs.is_empty() {
            return Ok(());
        }

        let match_direction = airway_legs.first().and_then(|leg| leg.from.identifier())
            == from_fix.identifier()
            && airway_legs.last().and_then(|leg| leg.to.identifier()) == to_fix.identifier();
        let match_reverse_direction = airway_legs.first().and_then(|leg| leg.from.identifier())
            == to_fix.identifier()
            && airway_legs.last().and_then(|leg| leg.to.identifier()) == from_fix.identifier();

        let airway_legs = if match_direction {
            airway_legs
        } else if match_reverse_direction {
            airway_legs
                .into_iter()
                .map(|leg| leg.into_reversed())
                .rev()
                .collect()
        } else {
            panic!("invalid result from navdata")
        };

        self.legs.extend(airway_legs.into_iter());
        Ok(())
    }

    fn handle_waypoint(&mut self, _value: &str, fix: AnyFix) -> Result<(), ParserError> {
        let last_fix = self.last_fix()?;
        if last_fix.latitude() == fix.latitude() && last_fix.longitude() == fix.longitude() {
            return Ok(());
        }
        self.legs.push(ResolvedLeg {
            identifier: None,
            from: last_fix,
            to: fix,
            direction_restriction: DirectionRestriction::None,
        });
        Ok(())
    }

    fn last_fix(&self) -> Result<AnyFix, ParserError> {
        self.last_fix_override
            .clone()
            .or_else(|| self.legs.last().map(|leg| leg.to.clone()))
            .ok_or(ParserError::MissingInitialFix)
    }
}
