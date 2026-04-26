use chrono::{DateTime, Utc};
use ulid::Ulid;

use crate::adapter::compat::{FlightPlan, Pilot, VatsimData};

#[derive(Debug, Clone)]
pub struct Flight {
    pub id: Ulid,
    pub cid: String,
    pub callsign: String,
    pub last_observed_at: DateTime<Utc>,
    pub departure: String,
    pub arrival: String,
    pub equipment: String,
    pub navigation_performance: String,
    pub transponder: String,
    pub raw_route: String,
    pub aircraft: String,
    pub altitude: i64,
    pub cruising_level: i64,
}

pub fn flights_from_vatsim(data: VatsimData) -> Vec<Flight> {
    data.pilots
        .into_iter()
        .chain(data.prefiles)
        .filter_map(map_pilot)
        .collect()
}

fn map_pilot(pilot: Pilot) -> Option<Flight> {
    let flight_plan = pilot.flight_plan?;
    let departure = flight_plan.departure.clone()?;
    let arrival = flight_plan.arrival.clone()?;
    if !is_china_airport(&departure) && !is_china_airport(&arrival) {
        return None;
    }
    if departure == arrival {
        return None;
    }
    if flight_plan.flight_rules.as_deref() != Some("I") {
        return None;
    }
    let aircraft = parse_aircraft(&flight_plan);

    Some(Flight {
        id: Ulid::new(),
        cid: pilot.cid.to_string(),
        callsign: pilot.callsign,
        last_observed_at: pilot.last_updated,
        departure,
        arrival,
        equipment: aircraft.equipment,
        navigation_performance: aircraft.navigation_performance,
        transponder: aircraft.transponder,
        raw_route: flight_plan.route.unwrap_or_default(),
        aircraft: aircraft.code,
        altitude: pilot.altitude.unwrap_or_default(),
        cruising_level: flight_plan
            .altitude
            .as_deref()
            .and_then(parse_flight_altitude)
            .unwrap_or_default(),
    })
}

fn is_china_airport(ident: &str) -> bool {
    let bytes = ident.as_bytes();
    ident.len() == 4
        && bytes[0] == b'Z'
        && matches!(
            bytes[1],
            b'B' | b'M' | b'S' | b'P' | b'G' | b'J' | b'Y' | b'W' | b'L' | b'U' | b'H'
        )
}

fn parse_flight_altitude(altitude: &str) -> Option<i64> {
    altitude.parse().ok().or_else(|| {
        altitude
            .strip_prefix("FL")
            .or_else(|| altitude.strip_prefix("fl"))
            .and_then(|level| level.parse::<i64>().ok())
            .map(|level| level * 100)
    })
}

struct AircraftParts {
    code: String,
    equipment: String,
    transponder: String,
    navigation_performance: String,
}

fn parse_aircraft(flight_plan: &FlightPlan) -> AircraftParts {
    let aircraft = flight_plan.aircraft.as_deref().unwrap_or_default();
    let aircraft_segments = aircraft.to_uppercase();
    let segments = aircraft_segments.split('/').collect::<Vec<_>>();
    let tail_segments = segments
        .get(1)
        .map(|segment| segment.split('-').collect::<Vec<_>>())
        .unwrap_or_default();

    AircraftParts {
        code: segments.first().copied().unwrap_or_default().to_owned(),
        equipment: tail_segments.get(1).copied().unwrap_or_default().to_owned(),
        transponder: segments.get(2).copied().unwrap_or_default().to_owned(),
        navigation_performance: pbn(flight_plan.remarks.as_deref().unwrap_or_default()),
    }
}

fn pbn(remarks: &str) -> String {
    let Some(start) = remarks.find("PBN/").map(|index| index + 4) else {
        return String::new();
    };
    let end = remarks[start..]
        .find(' ')
        .map(|index| start + index)
        .unwrap_or(remarks.len());
    remarks[start..end].to_owned()
}
