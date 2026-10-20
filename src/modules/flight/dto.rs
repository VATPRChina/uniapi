use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::modules::controller::models::CompatFutureController;
use crate::modules::flight::flight_plan::Segment;
use crate::modules::navdata::models::{
    AnyFix, ChinaMainlandBoundary, Fix, LegKind, ResolvedLeg, Sid,
};

use super::models::{CompatController, CompatPilot, CompatStatus, Flight};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MetarQuery {
    pub id: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct FlightRouteV2Query {
    /// Complete route, including the departure and arrival identifiers.
    pub route: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatVatprcStatusDto {
    pub last_updated: DateTime<Utc>,
    pub pilots: Vec<CompatPilotDto>,
    pub controllers: Vec<CompatControllerDto>,
    pub future_controllers: Vec<CompatFutureControllerDto>,
}

impl From<CompatStatus> for CompatVatprcStatusDto {
    fn from(status: CompatStatus) -> Self {
        Self {
            last_updated: status.last_updated,
            pilots: status.pilots.into_iter().map(Into::into).collect(),
            controllers: status.controllers.into_iter().map(Into::into).collect(),
            future_controllers: status
                .future_controllers
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatPilotDto {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub departure: Option<String>,
    pub arrival: Option<String>,
    pub aircraft: Option<String>,
}

impl From<CompatPilot> for CompatPilotDto {
    fn from(pilot: CompatPilot) -> Self {
        Self {
            cid: pilot.cid,
            name: pilot.name,
            callsign: pilot.callsign,
            departure: pilot.departure,
            arrival: pilot.arrival,
            aircraft: pilot.aircraft,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatControllerDto {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub frequency: String,
}

impl From<CompatController> for CompatControllerDto {
    fn from(controller: CompatController) -> Self {
        Self {
            cid: controller.cid,
            name: controller.name,
            callsign: controller.callsign,
            frequency: controller.frequency,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct CompatFutureControllerDto {
    pub callsign: String,
    pub name: String,
    pub start: String,
    pub start_utc: DateTime<Utc>,
    pub end: String,
    pub end_utc: DateTime<Utc>,
}

impl From<CompatFutureController> for CompatFutureControllerDto {
    fn from(controller: CompatFutureController) -> Self {
        Self {
            callsign: controller.callsign,
            name: controller.name,
            start: controller.start_at.format("%d %H:%M").to_string(),
            start_utc: controller.start_at,
            end: controller.end_at.format("%d %H:%M").to_string(),
            end_utc: controller.end_at,
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct TemporaryFlightQuery {
    pub departure: String,
    pub arrival: String,
    #[serde(default)]
    pub aircraft: String,
    #[serde(default)]
    pub flight_rules: String,
    #[serde(default)]
    pub wake_category: String,
    #[serde(default)]
    pub equipment: String,
    #[serde(default)]
    pub navigation_performance: String,
    #[serde(default)]
    pub transponder: String,
    #[serde(default)]
    pub raw_route: String,
    #[serde(default)]
    pub cruising_level: i64,
}

impl From<TemporaryFlightQuery> for Flight {
    fn from(query: TemporaryFlightQuery) -> Self {
        Self {
            id: Ulid::new(),
            cid: String::new(),
            callsign: String::new(),
            last_observed_at: Utc::now(),
            departure: query.departure,
            arrival: query.arrival,
            equipment: query.equipment,
            navigation_performance: query.navigation_performance,
            transponder: query.transponder,
            raw_route: query.raw_route,
            aircraft: query.aircraft,
            flight_rules: query.flight_rules,
            wake_category: query.wake_category,
            altitude: 0,
            cruising_level: query.cruising_level,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightDto {
    pub id: String,
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
    pub flight_rules: String,
    pub wake_category: String,
    pub altitude: i64,
    pub cruising_level: i64,
}

impl From<Flight> for FlightDto {
    fn from(flight: Flight) -> Self {
        Self {
            id: flight.id.to_string(),
            cid: flight.cid,
            callsign: flight.callsign,
            last_observed_at: flight.last_observed_at,
            departure: flight.departure,
            arrival: flight.arrival,
            equipment: flight.equipment,
            navigation_performance: flight.navigation_performance,
            transponder: flight.transponder,
            raw_route: flight.raw_route,
            aircraft: flight.aircraft,
            flight_rules: flight.flight_rules,
            wake_category: flight.wake_category,
            altitude: flight.altitude,
            cruising_level: flight.cruising_level,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct RouteDeliveryInfo {
    pub legs: Vec<FlightRouteLeg>,
    /// Mainland visits produced by the route segmenter, in route order.
    pub segments: Vec<FlightRouteSegment>,
    pub sid_candidates: Vec<SidCandidate>,
}

impl RouteDeliveryInfo {
    pub fn new(
        route: Vec<ResolvedLeg>,
        sid_candidates: Vec<SidCandidate>,
        segments: Vec<FlightRouteSegment>,
        boundary: &ChinaMainlandBoundary,
    ) -> Self {
        let legs: Vec<_> = route
            .into_iter()
            .map(|leg| FlightRouteLeg::from_leg(leg, boundary))
            .collect();
        Self {
            legs,
            segments,
            sid_candidates,
        }
    }
}

/// A mainland visit. Leg ranges refer to the top-level expanded legs array.
#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightRouteSegment {
    pub identifier: String,
    /// Inclusive zero-based leg index.
    pub start_index: usize,
    /// Exclusive zero-based leg index.
    pub end_index: usize,
    pub route_start_airport: Option<FlightRouteFix>,
    pub route_end_airport: Option<FlightRouteFix>,
    pub enroute_start_fix: FlightRouteFix,
    pub enroute_end_fix: FlightRouteFix,
    pub cross_border_enter_start_fix: Option<FlightRouteFix>,
    pub cross_border_enter_end_fix: Option<FlightRouteFix>,
    pub cross_border_exit_start_fix: Option<FlightRouteFix>,
    pub cross_border_exit_end_fix: Option<FlightRouteFix>,
}

impl From<&Segment<'_>> for FlightRouteSegment {
    fn from(segment: &Segment<'_>) -> Self {
        let from = FlightFix::from(&segment.route.legs.first().unwrap().from);
        let to = FlightFix::from(&segment.route.legs.last().unwrap().to);
        Self {
            identifier: format!("{}-{}", from.identifier, to.identifier),
            start_index: segment.route.start_index,
            end_index: segment.route.end_index(),
            route_start_airport: segment.get_route_start_airport().map(Into::into),
            route_end_airport: segment.get_route_end_airport().map(Into::into),
            enroute_start_fix: segment.get_enroute_start_fix().into(),
            enroute_end_fix: segment.get_enroute_end_fix().into(),
            cross_border_enter_start_fix: segment
                .get_cross_border_enter_start_fix()
                .map(Into::into),
            cross_border_enter_end_fix: segment.get_cross_border_enter_end_fix().map(Into::into),
            cross_border_exit_start_fix: segment.get_cross_border_exit_start_fix().map(Into::into),
            cross_border_exit_end_fix: segment.get_cross_border_exit_end_fix().map(Into::into),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SidCandidate {
    pub identifier: String,
    pub runway_transitions: Vec<String>,
    pub is_rnav: bool,
}

impl From<Sid> for SidCandidate {
    fn from(sid: Sid) -> Self {
        Self {
            identifier: sid.identifier.to_string(),
            runway_transitions: sid
                .proc
                .runway_transitions
                .into_iter()
                .map(|runway| runway.to_string())
                .sorted()
                .collect(),
            is_rnav: sid.proc.is_rnav,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightRouteLeg {
    /// Whether either endpoint is mainland or unresolved.
    pub is_mainland: bool,
    pub from: FlightRouteFix,
    pub to: FlightRouteFix,
    pub leg_identifier: String,
    pub is_unknown: bool,
    pub is_sid: bool,
    pub is_star: bool,
}

impl FlightRouteLeg {
    pub fn from_leg(leg: ResolvedLeg, boundary: &ChinaMainlandBoundary) -> Self {
        Self {
            is_mainland: leg.is_china(boundary),
            from: FlightRouteFix::from(&leg.from),
            to: FlightRouteFix::from(&leg.to),
            leg_identifier: leg.identifier.unwrap_or_default(),
            is_unknown: leg.is_unknown,
            is_sid: leg.kind == LegKind::Sid,
            is_star: leg.kind == LegKind::Star,
        }
    }
}

/// Coordinates are exposed only by the developer-only v2 route endpoint.
#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightRouteFix {
    pub identifier: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_unknown: bool,
}

impl From<&AnyFix> for FlightRouteFix {
    fn from(fix: &AnyFix) -> Self {
        let position = fix.position().filter(|(latitude, longitude)| {
            (-90.0..=90.0).contains(latitude) && (-180.0..=180.0).contains(longitude)
        });
        Self {
            identifier: FlightFix::from(fix).identifier,
            latitude: position.map(|point| point.0),
            longitude: position.map(|point| point.1),
            is_unknown: position.is_none(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightLeg {
    pub from: FlightFix,
    pub to: FlightFix,
    pub leg_identifier: String,
}

impl From<ResolvedLeg> for FlightLeg {
    fn from(leg: ResolvedLeg) -> Self {
        Self {
            from: FlightFix::from(&leg.from),
            to: FlightFix::from(&leg.to),
            leg_identifier: leg.identifier.unwrap_or_default(),
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct FlightFix {
    pub identifier: String,
}

impl From<&AnyFix> for FlightFix {
    fn from(fix: &AnyFix) -> Self {
        Self {
            identifier: match fix {
                AnyFix::GeoPoint(geo_point) => {
                    let latitude_minutes = (geo_point.latitude.abs() * 60.0).round() as u16;
                    let longitude_minutes = (geo_point.longitude.abs() * 60.0).round() as u16;
                    format!(
                        "{:02}{:02}{}{:03}{:02}{}",
                        latitude_minutes / 60,
                        latitude_minutes % 60,
                        if *geo_point.latitude < 0.0 { "S" } else { "N" },
                        longitude_minutes / 60,
                        longitude_minutes % 60,
                        if *geo_point.longitude < 0.0 { "W" } else { "E" },
                    )
                }
                _ => fix.identifier().unwrap_or_default().to_owned(),
            },
        }
    }
}

#[cfg(test)]
mod route_segment_tests {
    use super::*;
    use crate::modules::flight::flight_plan::RouteSegmenter;
    use crate::modules::navdata::models::{
        Airport, ChinaMainlandBoundary, DirectionRestriction, GeoPoint,
    };

    #[test]
    fn response_preserves_visit_ranges_and_known_crossing_endpoints() {
        let boundary = ChinaMainlandBoundary::from_rings(vec![vec![
            (100., 20.),
            (120., 20.),
            (120., 40.),
            (100., 40.),
        ]])
        .unwrap();
        let airport = |identifier: &str, longitude: f64| {
            AnyFix::Airport(Airport {
                identifier: identifier.try_into().unwrap(),
                latitude: 30.0.try_into().unwrap(),
                longitude: longitude.try_into().unwrap(),
            })
        };
        let outside: AnyFix = GeoPoint::new(30., 95.).unwrap().into();
        let inside: AnyFix = GeoPoint::new(30., 105.).unwrap().into();
        let foreign: AnyFix = GeoPoint::new(30., 125.).unwrap().into();
        let reentry: AnyFix = GeoPoint::new(30., 110.).unwrap().into();
        let fixes = [
            airport("RJAA", 140.),
            outside.clone(),
            AnyFix::Unknown("UNK1".into()),
            AnyFix::Unknown("UNK2".into()),
            inside.clone(),
            foreign,
            reentry,
            airport("ZAAA", 105.),
        ];
        let legs: Vec<_> = fixes
            .windows(2)
            .map(|pair| ResolvedLeg {
                from: pair[0].clone(),
                to: pair[1].clone(),
                identifier: None,
                is_unknown: pair.iter().any(AnyFix::is_unknown),
                kind: LegKind::Direct,
                direction_restriction: DirectionRestriction::None,
            })
            .collect();
        let segments = RouteSegmenter::new(&boundary)
            .segment(&legs)
            .iter()
            .map(FlightRouteSegment::from)
            .collect();
        let response = RouteDeliveryInfo::new(legs, vec![], segments, &boundary);
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["legs"].as_array().unwrap().len(), 7);
        assert_eq!(json["legs"][0]["is_mainland"], false);
        assert_eq!(json["legs"][1]["is_mainland"], true);
        let segments = json["segments"].as_array().unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0]["start_index"], 0);
        assert_eq!(segments[0]["end_index"], 5);
        assert_eq!(segments[1]["start_index"], 5);
        assert_eq!(segments[1]["end_index"], 7);
        assert_eq!(segments[0]["route_start_airport"]["identifier"], "RJAA");
        assert!(segments[0]["route_end_airport"].is_null());
        assert_eq!(
            segments[0]["cross_border_enter_start_fix"]["identifier"],
            FlightFix::from(&outside).identifier
        );
        assert_eq!(
            segments[0]["cross_border_enter_end_fix"]["identifier"],
            FlightFix::from(&inside).identifier
        );
        assert_eq!(segments[1]["route_end_airport"]["identifier"], "ZAAA");
        assert!(segments[1]["cross_border_exit_start_fix"].is_null());
    }
}
