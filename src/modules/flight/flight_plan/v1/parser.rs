use crate::modules::flight::flight_plan::RouteToken;
use crate::modules::flight::flight_plan::v1::lexer::{LexerError, lex_route};
use crate::modules::navdata::models::{AnyFix, DirectionRestriction, LegKind, ResolvedLeg};
use crate::modules::navdata::service::{InvalidNavdataError, NavdataService};

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("navdata error: {0}")]
    Navdata(#[from] InvalidNavdataError),
    #[error("lexer error: {0}")]
    Lexer(#[from] LexerError),
    #[error("no initial fix in route")]
    MissingInitialFix,
}

pub async fn parse_route(
    navdata: &NavdataService,
    route: &str,
) -> Result<Vec<ResolvedLeg>, ParserError> {
    tracing::info!("parsing route: {}", route);
    let tokens = lex_route(navdata, route).await?;
    RouteParser::new(navdata, tokens).parse().await
}

struct RouteParser<'a> {
    navdata: &'a NavdataService,
    tokens: Vec<RouteToken>,
    legs: Vec<ResolvedLeg>,
    last_fix_override: Option<AnyFix>,
}

impl<'a> RouteParser<'a> {
    fn new(navdata: &'a NavdataService, tokens: Vec<RouteToken>) -> Self {
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
                RouteToken::Unknown { value } => self.handle_unknown_waypoint(&value)?,
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

        let (Some(first), Some(last)) = (airway_legs.first(), airway_legs.last()) else {
            return Ok(());
        };
        let match_direction = &first.from == from_fix && &last.to == to_fix;
        let match_reverse_direction = &first.from == to_fix && &last.to == from_fix;

        let airway_legs = if match_direction {
            airway_legs
        } else if match_reverse_direction {
            airway_legs
                .into_iter()
                .map(|leg| leg.into_reversed())
                .rev()
                .collect()
        } else {
            tracing::warn!(
                airway = ident,
                from = from_value,
                to = to_value,
                "airway lookup returned inconsistent endpoints; treating leg as direct"
            );
            return Ok(());
        };

        self.legs.extend(airway_legs);
        Ok(())
    }

    fn handle_waypoint(&mut self, _value: &str, fix: AnyFix) -> Result<(), ParserError> {
        let last_fix = self.last_fix()?;
        if last_fix == fix {
            return Ok(());
        }
        self.legs.push(ResolvedLeg {
            identifier: None,
            from: last_fix,
            to: fix,
            is_unknown: false,
            kind: LegKind::Direct,
            direction_restriction: DirectionRestriction::None,
        });
        Ok(())
    }

    fn handle_unknown_waypoint(&mut self, value: &str) -> Result<(), ParserError> {
        self.handle_waypoint(value, AnyFix::Unknown(value.to_owned()))
    }

    fn last_fix(&self) -> Result<AnyFix, ParserError> {
        self.last_fix_override
            .clone()
            .or_else(|| self.legs.last().map(|leg| leg.to.clone()))
            .ok_or(ParserError::MissingInitialFix)
    }
}
