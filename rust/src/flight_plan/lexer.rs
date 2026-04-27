use sqlx::PgPool;

use crate::{
    flight_plan::{Fix, FixKind, RouteToken},
    repository::{airport, airway, ndb_navaid, procedure, vhf_navaid, waypoint},
};

#[derive(Debug, thiserror::Error)]
pub enum LexerError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub async fn lex_route(db: &PgPool, route: &str) -> Result<Vec<RouteToken>, LexerError> {
    let mut lexer = RouteLexer::new(route);
    lexer.parse_all_segments(db).await?;
    Ok(lexer.tokens)
}

struct RouteLexer {
    tokens: Vec<RouteToken>,
    current_lat: f64,
    current_lon: f64,
}

impl RouteLexer {
    fn new(route: &str) -> Self {
        let tokens = route
            .split(' ')
            .map(|segment| {
                let value = segment
                    .split_once('/')
                    .map(|(head, _)| head)
                    .unwrap_or(segment)
                    .trim()
                    .to_owned();
                RouteToken::Unknown { value }
            })
            .collect();

        Self {
            tokens,
            current_lat: 0.0,
            current_lon: 0.0,
        }
    }

    async fn parse_all_segments(&mut self, db: &PgPool) -> Result<(), LexerError> {
        for index in 0..self.tokens.len() {
            self.parse_segment(db, index, true).await?;
        }

        self.current_lat = 0.0;
        self.current_lon = 0.0;
        for index in 0..self.tokens.len() {
            self.parse_segment(db, index, false).await?;
        }
        Ok(())
    }

    async fn parse_segment(
        &mut self,
        db: &PgPool,
        index: usize,
        skip_need_next: bool,
    ) -> Result<(), LexerError> {
        if !matches!(self.tokens[index], RouteToken::Unknown { .. }) {
            return Ok(());
        }

        if self.resolve_initial_speed_and_altitude(index) {
            return Ok(());
        }
        if self.resolve_airport(db, index).await? {
            return Ok(());
        }
        if self.resolve_sid(db, index).await? {
            return Ok(());
        }
        if !skip_need_next && self.resolve_star(db, index).await? {
            return Ok(());
        }
        if !skip_need_next && self.resolve_airway(db, index).await? {
            return Ok(());
        }
        if !skip_need_next && self.resolve_dct(index) {
            return Ok(());
        }
        if self.resolve_waypoint(db, index).await? {
            return Ok(());
        }
        if self.resolve_geo7(index) || self.resolve_geo11(index) {
            return Ok(());
        }
        if self.resolve_airport_fallback(index) {
            return Ok(());
        }
        if self.resolve_sid_fallback(index) {
            return Ok(());
        }
        if !skip_need_next && self.resolve_star_fallback(index) {
            return Ok(());
        }
        if !skip_need_next {
            self.resolve_airway_fallback(index);
        }
        Ok(())
    }

    fn value(&self, index: usize) -> &str {
        self.tokens[index].value()
    }

    fn resolve_initial_speed_and_altitude(&mut self, index: usize) -> bool {
        if index != 0 {
            return false;
        }
        let token = self.value(index);
        let Some(speed_len) = cruise_speed_len(token) else {
            return false;
        };
        if cruise_altitude_len(&token[speed_len..]).is_none() {
            return false;
        }
        self.tokens[index] = RouteToken::SpeedAndAltitude {
            value: token.to_owned(),
        };
        true
    }

    async fn resolve_airport(&mut self, db: &PgPool, index: usize) -> Result<bool, LexerError> {
        if !self.is_airport_position(index) {
            return Ok(false);
        }
        let Some(airport) = airport::find(db, self.value(index)).await? else {
            return Ok(false);
        };
        self.current_lat = airport.latitude;
        self.current_lon = airport.longitude;
        self.tokens[index] = RouteToken::Fix {
            value: airport.identifier.clone().unwrap_or_default(),
            fix: airport,
        };
        Ok(true)
    }

    async fn resolve_sid(&mut self, db: &PgPool, index: usize) -> Result<bool, LexerError> {
        let Some(RouteToken::Fix { fix, value, .. }) = self.last(index) else {
            return Ok(false);
        };
        if fix.kind != FixKind::Airport {
            return Ok(false);
        }
        let Some(proc_ident) = procedure::find_sid(db, self.value(index), value).await? else {
            return Ok(false);
        };
        self.tokens[index] = RouteToken::SidLeg {
            value: proc_ident.clone(),
            procedure: Some(proc_ident),
        };
        Ok(true)
    }

    async fn resolve_star(&mut self, db: &PgPool, index: usize) -> Result<bool, LexerError> {
        let Some(RouteToken::Fix { fix, value, .. }) = self.next(index) else {
            return Ok(false);
        };
        if fix.kind != FixKind::Airport {
            return Ok(false);
        }
        let Some(proc_ident) = procedure::find_star(db, self.value(index), value).await? else {
            return Ok(false);
        };
        self.tokens[index] = RouteToken::StarLeg {
            value: proc_ident.clone(),
            procedure: Some(proc_ident),
        };
        Ok(true)
    }

    async fn resolve_airway(&mut self, db: &PgPool, index: usize) -> Result<bool, LexerError> {
        let (Some(last), Some(next)) = (self.last(index), self.next(index)) else {
            return Ok(false);
        };
        if !last.is_fix() || !next.is_fix() {
            return Ok(false);
        }
        let exists_left = airway::exists_with_fix(db, self.value(index), last.value()).await?;
        let exists_right = airway::exists_with_fix(db, self.value(index), next.value()).await?;
        if !exists_left || !exists_right {
            return Ok(false);
        }
        self.tokens[index] = RouteToken::AirwayLeg {
            value: self.value(index).to_owned(),
        };
        Ok(true)
    }

    fn resolve_dct(&mut self, index: usize) -> bool {
        if self.value(index) != "DCT"
            || !self.last(index).is_some_and(RouteToken::is_fix)
            || !self.next(index).is_some_and(RouteToken::is_fix)
        {
            return false;
        }
        self.tokens[index] = RouteToken::DirectLeg {
            value: "DCT".to_owned(),
        };
        true
    }

    async fn resolve_waypoint(&mut self, db: &PgPool, index: usize) -> Result<bool, LexerError> {
        let Some(fix) = find_fix(db, self.value(index), self.current_lat, self.current_lon).await?
        else {
            return Ok(false);
        };
        self.tokens[index] = RouteToken::Fix {
            value: fix
                .identifier
                .clone()
                .unwrap_or_else(|| self.value(index).to_owned()),
            fix,
        };
        Ok(true)
    }

    fn resolve_geo7(&mut self, index: usize) -> bool {
        let token = self.value(index);
        if token.len() != 7 {
            return false;
        }
        let bytes = token.as_bytes();
        if !matches!(bytes[2], b'N' | b'S') || !matches!(bytes[6], b'E' | b'W') {
            return false;
        }
        let Ok(lat) = token[..2].parse::<f64>() else {
            return false;
        };
        let Ok(lon) = token[3..6].parse::<f64>() else {
            return false;
        };
        self.set_geo(index, lat, bytes[2], lon, bytes[6]);
        true
    }

    fn resolve_geo11(&mut self, index: usize) -> bool {
        let token = self.value(index);
        if token.len() != 11 {
            return false;
        }
        let bytes = token.as_bytes();
        if !matches!(bytes[4], b'N' | b'S') || !matches!(bytes[10], b'E' | b'W') {
            return false;
        }
        let Ok(lat_deg) = token[..2].parse::<f64>() else {
            return false;
        };
        let Ok(lat_min) = token[2..4].parse::<f64>() else {
            return false;
        };
        let Ok(lon_deg) = token[5..8].parse::<f64>() else {
            return false;
        };
        let Ok(lon_min) = token[8..10].parse::<f64>() else {
            return false;
        };
        self.set_geo(
            index,
            lat_deg + lat_min / 60.0,
            bytes[4],
            lon_deg + lon_min / 60.0,
            bytes[10],
        );
        true
    }

    fn set_geo(&mut self, index: usize, lat: f64, lat_sign: u8, lon: f64, lon_sign: u8) {
        let lat = lat * if lat_sign == b'N' { 1.0 } else { -1.0 };
        let lon = lon * if lon_sign == b'E' { 1.0 } else { -1.0 };
        self.current_lat = lat;
        self.current_lon = lon;
        self.tokens[index] = RouteToken::Fix {
            value: self.value(index).to_owned(),
            fix: Fix::geo(lat, lon),
        };
    }

    fn resolve_airport_fallback(&mut self, index: usize) -> bool {
        if !self.is_airport_position(index) {
            return false;
        }
        self.tokens[index] = RouteToken::Fix {
            value: self.value(index).to_owned(),
            fix: Fix::identified(FixKind::Airport, "", self.value(index), 0.0, 0.0),
        };
        true
    }

    fn resolve_sid_fallback(&mut self, index: usize) -> bool {
        if index != 1 || !self.next(index).is_some_and(RouteToken::is_fix) {
            return false;
        }
        let Some(RouteToken::Fix { fix, .. }) = self.last(index) else {
            return false;
        };
        if fix.kind != FixKind::Airport {
            return false;
        }
        self.tokens[index] = RouteToken::SidLeg {
            value: self.value(index).to_owned(),
            procedure: None,
        };
        true
    }

    fn resolve_star_fallback(&mut self, index: usize) -> bool {
        if index + 2 != self.tokens.len() || !self.last(index).is_some_and(RouteToken::is_fix) {
            return false;
        }
        let Some(RouteToken::Fix { fix, .. }) = self.next(index) else {
            return false;
        };
        if fix.kind != FixKind::Airport {
            return false;
        }
        self.tokens[index] = RouteToken::StarLeg {
            value: self.value(index).to_owned(),
            procedure: None,
        };
        true
    }

    fn resolve_airway_fallback(&mut self, index: usize) -> bool {
        let value = self.value(index);
        if !self.last(index).is_some_and(RouteToken::is_fix)
            || !self.next(index).is_some_and(RouteToken::is_fix)
            || value.len() < 2
            || !value.as_bytes()[0].is_ascii_alphabetic()
            || !value.as_bytes()[1..].iter().all(u8::is_ascii_digit)
        {
            return false;
        }
        self.tokens[index] = RouteToken::AirwayLeg {
            value: value.to_owned(),
        };
        true
    }

    fn is_airport_position(&self, index: usize) -> bool {
        index == 0
            || (index == 1 && matches!(self.last(index), Some(RouteToken::SpeedAndAltitude { .. })))
            || index + 1 == self.tokens.len()
    }

    fn last(&self, index: usize) -> Option<&RouteToken> {
        index
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
    }

    fn next(&self, index: usize) -> Option<&RouteToken> {
        self.tokens.get(index + 1)
    }
}

fn cruise_speed_len(token: &str) -> Option<usize> {
    let token = token.as_bytes();
    if token.len() >= 4
        && matches!(token[0], b'N' | b'K')
        && token[1..4].iter().all(u8::is_ascii_digit)
    {
        return Some(4);
    }
    if token.len() >= 5 && token[0] == b'M' && token[1..5].iter().all(u8::is_ascii_digit) {
        return Some(5);
    }
    None
}

fn cruise_altitude_len(token: &str) -> Option<usize> {
    let bytes = token.as_bytes();
    if bytes.len() == 4 && matches!(bytes[0], b'F' | b'S' | b'A' | b'M') {
        return bytes[1..].iter().all(u8::is_ascii_digit).then_some(4);
    }
    if bytes.len() == 5 && bytes[0] == b'M' {
        return bytes[1..].iter().all(u8::is_ascii_digit).then_some(5);
    }
    if bytes.len() == 3 && bytes == b"VFR" {
        return Some(3);
    }
    None
}

async fn find_fix(
    db: &PgPool,
    ident: &str,
    lat: f64,
    lon: f64,
) -> Result<Option<Fix>, sqlx::Error> {
    let mut fixes = waypoint::find(db, ident).await?;
    fixes.extend(vhf_navaid::find(db, ident).await?);
    fixes.extend(ndb_navaid::find(db, ident).await?);

    Ok(fixes.into_iter().min_by(|left, right| {
        let left_distance = squared_distance(left, lat, lon);
        let right_distance = squared_distance(right, lat, lon);
        left_distance.total_cmp(&right_distance)
    }))
}

fn squared_distance(fix: &Fix, lat: f64, lon: f64) -> f64 {
    ((lat - fix.latitude) * (lat - fix.latitude)) + ((lon - fix.longitude) * (lon - fix.longitude))
}
