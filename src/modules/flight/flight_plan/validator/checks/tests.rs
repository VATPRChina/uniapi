use super::*;
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::navdata::models::{
    Airport, AnyFix, DirectionRestriction, GeoPoint, LegKind, LevelRestrictionType, ResolvedLeg,
    Waypoint, WaypointKind,
};

async fn navdata() -> NavdataService {
    NavdataService::with_preferred_routes_path(":memory:", "assets/test/routes.csv")
        .await
        .unwrap()
}

fn flight() -> Flight {
    Flight {
        id: ulid::Ulid::nil(),
        cid: "1234567".into(),
        callsign: "CCA123".into(),
        last_observed_at: chrono::Utc::now(),
        departure: "ZBAA".into(),
        arrival: "ZSPD".into(),
        equipment: "RW".into(),
        navigation_performance: "D1".into(),
        transponder: "S".into(),
        raw_route: String::new(),
        aircraft: "A320".into(),
        flight_rules: "I".into(),
        wake_category: "M".into(),
        altitude: 0,
        cruising_level: 33100,
    }
}

fn fix(icao: &str) -> AnyFix {
    AnyFix::Waypoint(Waypoint {
        icao_code: icao.try_into().unwrap(),
        identifier: "FIX".try_into().unwrap(),
        latitude: 30.0.try_into().unwrap(),
        longitude: 110.0.try_into().unwrap(),
        kind: WaypointKind::Enroute,
    })
}

fn leg(kind: LegKind) -> ResolvedLeg {
    ResolvedLeg {
        from: fix("ZB"),
        to: fix("ZS"),
        identifier: if kind == LegKind::Direct {
            None
        } else {
            Some("A1".into())
        },
        is_unknown: false,
        kind,
        direction_restriction: DirectionRestriction::None,
    }
}

fn validate<V: Validator>(
    navdata: &NavdataService,
    flight: &Flight,
    legs: Vec<ResolvedLeg>,
    preferred_route: Option<&PreferredRoute>,
) -> ValidatorResult {
    let result = V::validate(
        &ValidationContext {
            flight,
            route: &ParsedRoute { legs },
            preferred_route,
            preferred_routes: &[],
        },
        navdata,
    );
    assert_eq!(result.validator_ident, V::IDENT);
    assert_eq!(result.field, V::FIELD);
    result
}

fn preferred() -> PreferredRoute {
    PreferredRoute {
        name: "Test".into(),
        departure: "ZBAA".into(),
        arrival: "ZSPD".into(),
        raw_route: "DCT".into(),
        cruising_level_restriction: LevelRestrictionType::Standard,
        allowed_altitudes: Vec::new(),
        minimal_altitude: 0,
        remarks: String::new(),
        valid_from: None,
        valid_until: None,
        is_public: true,
    }
}

#[tokio::test]
async fn rvsm_checks_capability_only_in_the_rvsm_band() {
    let navdata = navdata().await;
    let mut flight = flight();
    for level in [29100, 30000, 33100, 41100] {
        flight.cruising_level = level;
        flight.equipment = "rw".into();
        assert_eq!(
            validate::<EquipmentRvsmValidator>(&navdata, &flight, vec![], None).status,
            ValidatorStatus::Pass
        );
        flight.equipment = "R".into();
        let result = validate::<EquipmentRvsmValidator>(&navdata, &flight, vec![], None);
        assert_eq!(result.status, ValidatorStatus::Rejected);
        assert_eq!(
            result.warnings,
            vec![WarningMessage::new(
                WarningMessageField::Equipment,
                WarningMessageCode::NoRvsm
            )]
        );
    }
    for level in [29099, 41101] {
        flight.cruising_level = level;
        let result = validate::<EquipmentRvsmValidator>(&navdata, &flight, vec![], None);
        assert_eq!(result.status, ValidatorStatus::Suppressed);
        assert!(result.warnings.is_empty());
    }
    flight.cruising_level = 0;
    assert_eq!(
        validate::<EquipmentRvsmValidator>(&navdata, &flight, vec![], None).status,
        ValidatorStatus::Unavailable
    );
    flight.cruising_level = 33100;
    flight.equipment.clear();
    assert_eq!(
        validate::<EquipmentRvsmValidator>(&navdata, &flight, vec![], None).status,
        ValidatorStatus::Unavailable
    );
}

#[tokio::test]
async fn rnav1_requires_equipment_r_and_accepts_all_rnav1_pbn_codes() {
    let navdata = navdata().await;
    let mut flight = flight();
    flight.equipment = "r".into();
    for pbn in ["D1", "D2", "D3", "D4", " b1d2s1 ", " b1d3s1 ", "b1d4s1"] {
        flight.navigation_performance = pbn.into();
        assert_eq!(
            validate::<EquipmentRnav1Validator>(&navdata, &flight, vec![], None).status,
            ValidatorStatus::Pass
        );
    }
    flight.equipment = "W".into();
    flight.navigation_performance = "D1".into();
    let result = validate::<EquipmentRnav1Validator>(&navdata, &flight, vec![], None);
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(result.warnings[0].field, WarningMessageField::Equipment);
    flight.equipment = "R".into();
    for pbn in ["", "B1", "C1", "D5", "D"] {
        flight.navigation_performance = pbn.into();
        let result = validate::<EquipmentRnav1Validator>(&navdata, &flight, vec![], None);
        assert_eq!(result.status, ValidatorStatus::Rejected);
        assert_eq!(
            result.warnings,
            vec![WarningMessage::new(
                WarningMessageField::NavigationPerformance,
                WarningMessageCode::NoRnav1
            )]
        );
    }
    flight.equipment = "W".into();
    assert_eq!(
        validate::<EquipmentRnav1Validator>(&navdata, &flight, vec![], None)
            .warnings
            .len(),
        2
    );
    flight.equipment.clear();
    flight.navigation_performance = "D4".into();
    let result = validate::<EquipmentRnav1Validator>(&navdata, &flight, vec![], None);
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(
        result.warnings,
        vec![WarningMessage::new(
            WarningMessageField::Equipment,
            WarningMessageCode::NoRnav1,
        )]
    );
    flight.navigation_performance.clear();
    let result = validate::<EquipmentRnav1Validator>(&navdata, &flight, vec![], None);
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(
        result.warnings,
        vec![
            WarningMessage::new(WarningMessageField::Equipment, WarningMessageCode::NoRnav1),
            WarningMessage::new(
                WarningMessageField::NavigationPerformance,
                WarningMessageCode::NoRnav1
            ),
        ]
    );
}

#[tokio::test]
async fn direct_legs_flag_domestic_segments_but_skip_cross_border_airports_and_procedures() {
    let navdata = navdata().await;
    let flight = flight();
    let mut airport = leg(LegKind::Direct);
    airport.from = AnyFix::Airport(Airport {
        identifier: "ZBAA".try_into().unwrap(),
        latitude: 40.0.try_into().unwrap(),
        longitude: 116.0.try_into().unwrap(),
    });
    let mut foreign = leg(LegKind::Direct);
    foreign.to = fix("RJ");
    let mut sid = leg(LegKind::Sid);
    sid.identifier = None;
    let mut inbound = leg(LegKind::Direct);
    inbound.from = fix("RJ");
    let cross_border = validate::<RouteDirectLegValidator>(
        &navdata,
        &flight,
        vec![foreign.clone(), inbound],
        None,
    );
    assert_eq!(cross_border.status, ValidatorStatus::Pass);
    assert!(cross_border.warnings.is_empty());
    let legs = vec![airport, foreign, sid, leg(LegKind::Direct)];
    let result = validate::<RouteDirectLegValidator>(&navdata, &flight, legs.clone(), None);
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(
        result.warnings,
        vec![expected_route_warning(
            3,
            WarningMessageCode::RouteDirectSegment,
            &legs[3]
        )]
    );
    let mut geo = leg(LegKind::Direct);
    geo.to = AnyFix::GeoPoint(GeoPoint::new(30., 111.).unwrap());
    assert_eq!(
        validate::<RouteDirectLegValidator>(&navdata, &flight, vec![geo], None).status,
        ValidatorStatus::Rejected
    );
}

#[tokio::test]
async fn unknown_check_reports_unknown_connections_and_both_endpoints() {
    let navdata = navdata().await;
    let flight = flight();
    let mut connection = leg(LegKind::Airway);
    connection.is_unknown = true;
    let mut from = leg(LegKind::Airway);
    from.from = AnyFix::Unknown("MISSING".into());
    let mut to = leg(LegKind::Airway);
    to.to = AnyFix::Unknown("MISSING".into());
    let legs = vec![leg(LegKind::Airway), connection, from, to];
    let result = validate::<RouteUnknownLegOrFixValidator>(&navdata, &flight, legs.clone(), None);
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(
        result.warnings,
        (1..=3)
            .map(|index| {
                expected_route_warning(
                    index,
                    WarningMessageCode::RouteUnknownLegOrFix,
                    &legs[index],
                )
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        result
            .warnings
            .iter()
            .map(|w| w.field_index)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3)]
    );
    assert!(
        result
            .warnings
            .iter()
            .all(|w| w.message_code == WarningMessageCode::RouteUnknownLegOrFix)
    );
}

#[tokio::test]
async fn approval_flags_v_and_x_airways_touching_china() {
    let navdata = navdata().await;
    let flight = flight();
    let mut legs = vec![];
    for ident in ["A1", "V1", "x2"] {
        let mut airway = leg(LegKind::Airway);
        airway.identifier = Some(ident.into());
        legs.push(airway);
    }
    let mut foreign = leg(LegKind::Airway);
    foreign.identifier = Some("V1".into());
    foreign.from = fix("RJ");
    legs.push(foreign);
    let mut sid = leg(LegKind::Sid);
    sid.identifier = Some("V1".into());
    legs.push(sid);
    let result = validate::<AirwayApprovalValidator>(&navdata, &flight, legs.clone(), None);
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(
        result.warnings,
        vec![
            expected_route_warning(1, WarningMessageCode::AirwayRequireApproval, &legs[1]),
            expected_route_warning(2, WarningMessageCode::AirwayRequireApproval, &legs[2]),
            expected_route_warning(3, WarningMessageCode::AirwayRequireApproval, &legs[3]),
        ]
    );
}

#[tokio::test]
async fn direction_flags_backward_airways_and_clears_after_reversal() {
    let navdata = navdata().await;
    let flight = flight();
    let mut backward = leg(LegKind::Airway);
    backward.direction_restriction = DirectionRestriction::Backward;
    let mut sid = leg(LegKind::Sid);
    sid.direction_restriction = DirectionRestriction::Backward;
    let result = validate::<AirwayDirectionValidator>(
        &navdata,
        &flight,
        vec![leg(LegKind::Airway), backward.clone(), sid],
        None,
    );
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(
        result.warnings,
        vec![expected_route_warning(
            1,
            WarningMessageCode::RouteLegDirection,
            &backward
        )]
    );
    assert_eq!(
        validate::<AirwayDirectionValidator>(
            &navdata,
            &flight,
            vec![backward.into_reversed()],
            None
        )
        .status,
        ValidatorStatus::Pass
    );
}

#[tokio::test]
async fn route_checks_handle_empty_clean_and_preferred_routes() {
    let navdata = navdata().await;
    let flight = flight();
    let preferred = preferred();
    macro_rules! check {
        ($validator:ty, $suppressed:expr) => {{
            assert_eq!(
                validate::<$validator>(&navdata, &flight, vec![], None).status,
                ValidatorStatus::Unavailable
            );
            let result =
                validate::<$validator>(&navdata, &flight, vec![leg(LegKind::Airway)], None);
            assert_eq!(result.status, ValidatorStatus::Pass);
            assert!(result.warnings.is_empty());
            let result = validate::<$validator>(
                &navdata,
                &flight,
                vec![leg(LegKind::Direct), {
                    let mut restricted = leg(LegKind::Airway);
                    restricted.identifier = Some("V1".into());
                    restricted.direction_restriction = DirectionRestriction::Backward;
                    restricted
                }],
                Some(&preferred),
            );
            assert_eq!(
                result.status,
                if $suppressed {
                    ValidatorStatus::Suppressed
                } else {
                    ValidatorStatus::Pass
                }
            );
            assert!(result.warnings.is_empty());
        }};
    }
    check!(RouteDirectLegValidator, true);
    check!(RouteUnknownLegOrFixValidator, true);
    check!(AirwayApprovalValidator, true);
    check!(AirwayDirectionValidator, true);
}

fn airport(identifier: &str) -> AnyFix {
    AnyFix::Airport(Airport {
        identifier: identifier.try_into().unwrap(),
        latitude: 40.0.try_into().unwrap(),
        longitude: 116.0.try_into().unwrap(),
    })
}

#[tokio::test]
async fn airport_checks_match_resolved_departure_and_arrival_endpoints() {
    let navdata = navdata().await;
    let mut flight = flight();
    flight.departure = " zbaa ".into();
    flight.arrival = "zspd".into();
    let mut first = leg(LegKind::Direct);
    first.from = airport("ZBAA");
    let mut last = leg(LegKind::Direct);
    last.to = airport("ZSPD");
    let route = vec![first, leg(LegKind::Airway), last];
    for result in [
        validate::<DepartureAirportValidator>(&navdata, &flight, route.clone(), None),
        validate::<ArrivalAirportValidator>(&navdata, &flight, route, None),
    ] {
        assert_eq!(result.status, ValidatorStatus::Pass);
        assert!(result.warnings.is_empty());
    }
}

#[tokio::test]
async fn airport_checks_reject_unknown_non_airport_and_mismatched_endpoints() {
    let navdata = navdata().await;
    let flight = flight();
    for endpoint in [AnyFix::Unknown("ZBAA".into()), fix("ZB"), airport("ZGGG")] {
        let mut route_leg = leg(LegKind::Direct);
        route_leg.from = endpoint.clone();
        route_leg.to = endpoint;
        for (result, identifier, field) in [
            (
                validate::<DepartureAirportValidator>(
                    &navdata,
                    &flight,
                    vec![route_leg.clone()],
                    None,
                ),
                "ZBAA",
                WarningMessageField::Departure,
            ),
            (
                validate::<ArrivalAirportValidator>(&navdata, &flight, vec![route_leg], None),
                "ZSPD",
                WarningMessageField::Arrival,
            ),
        ] {
            assert_eq!(result.status, ValidatorStatus::Rejected);
            assert_eq!(
                result.warnings,
                vec![WarningMessage::with_parameter(
                    field,
                    WarningMessageCode::InvalidAirport,
                    identifier
                )]
            );
        }
    }
}

#[tokio::test]
async fn airport_checks_distinguish_missing_inputs_from_invalid_identifiers() {
    let navdata = navdata().await;
    let mut flight = flight();
    for identifier in ["ZB", "ZBAAA", "1234", "ZB1A", "ZB!A", "机场机场"] {
        flight.departure = identifier.into();
        flight.arrival = identifier.into();
        assert_eq!(
            validate::<DepartureAirportValidator>(&navdata, &flight, vec![], None).status,
            ValidatorStatus::Rejected
        );
        assert_eq!(
            validate::<ArrivalAirportValidator>(&navdata, &flight, vec![], None).status,
            ValidatorStatus::Rejected
        );
    }
    for identifier in ["", " ", "ZBAA"] {
        flight.departure = identifier.into();
        flight.arrival = identifier.into();
        for result in [
            validate::<DepartureAirportValidator>(&navdata, &flight, vec![], None),
            validate::<ArrivalAirportValidator>(&navdata, &flight, vec![], None),
        ] {
            assert_eq!(result.status, ValidatorStatus::Unavailable);
            assert!(result.warnings.is_empty());
        }
    }
}

#[tokio::test]
async fn direct_and_approval_checks_skip_fully_foreign_segments() {
    let navdata = navdata().await;
    let flight = flight();
    let mut direct = leg(LegKind::Direct);
    direct.from = fix("RJ");
    direct.to = fix("RK");
    assert_eq!(
        validate::<RouteDirectLegValidator>(&navdata, &flight, vec![direct.clone()], None).status,
        ValidatorStatus::Pass
    );
    direct.kind = LegKind::Airway;
    direct.identifier = Some("V1".into());
    assert_eq!(
        validate::<AirwayApprovalValidator>(&navdata, &flight, vec![direct], None).status,
        ValidatorStatus::Pass
    );
}

#[tokio::test]
async fn preferred_route_reports_public_matches_without_exposing_private_routes() {
    let navdata = navdata().await;
    let mut flight = flight();
    flight.raw_route = "SUBMITTED DCT ROUTE".into();
    let mut preferred = preferred();
    preferred.raw_route = "DESIGNATED ROUTE".into();
    for public in [true, false] {
        preferred.is_public = public;
        let result = validate::<PreferredRouteValidator>(
            &navdata,
            &flight,
            vec![leg(LegKind::Airway)],
            Some(&preferred),
        );
        assert_eq!(result.status, ValidatorStatus::Pass);
        assert_eq!(
            result.warnings,
            vec![WarningMessage::with_parameter(
                WarningMessageField::Route,
                WarningMessageCode::RouteMatchPreferred,
                if public {
                    "DESIGNATED ROUTE"
                } else {
                    "SUBMITTED DCT ROUTE"
                },
            )]
        );
    }
}

#[tokio::test]
async fn preferred_route_distinguishes_missing_routes_from_unmatched_candidates() {
    let navdata = navdata().await;
    let flight = flight();
    let route = ParsedRoute {
        legs: vec![leg(LegKind::Airway)],
    };
    let mut first = preferred();
    first.name = "01".into();
    first.raw_route = "FIRST ROUTE".into();
    let mut second = preferred();
    second.name = "02".into();
    second.raw_route = "SECOND ROUTE".into();
    let mut private = preferred();
    private.is_public = false;
    private.raw_route = "PRIVATE ROUTE".into();
    for (candidates, status, parameter) in [
        (vec![], ValidatorStatus::Suppressed, None),
        (
            vec![&second, &private, &first],
            ValidatorStatus::Rejected,
            Some("FIRST ROUTE,SECOND ROUTE"),
        ),
        (vec![&private], ValidatorStatus::Rejected, Some("")),
    ] {
        let result = PreferredRouteValidator::validate(
            &ValidationContext {
                flight: &flight,
                route: &route,
                preferred_route: None,
                preferred_routes: &candidates,
            },
            &navdata,
        );
        assert_eq!(result.validator_ident, PreferredRouteValidator::IDENT);
        assert_eq!(result.field, WarningMessageField::Route);
        assert_eq!(result.status, status);
        assert_eq!(
            result.warnings,
            parameter
                .map(|parameter| WarningMessage::with_parameter(
                    WarningMessageField::Route,
                    WarningMessageCode::NotPreferredRoute,
                    parameter,
                ))
                .into_iter()
                .collect::<Vec<_>>()
        );
    }
    let result = validate::<PreferredRouteValidator>(&navdata, &flight, vec![], Some(&first));
    assert_eq!(result.status, ValidatorStatus::Unavailable);
    assert!(result.warnings.is_empty());
}

fn expected_route_warning(
    index: usize,
    code: WarningMessageCode,
    leg: &ResolvedLeg,
) -> WarningMessage {
    WarningMessage {
        message_code: code,
        field: WarningMessageField::Route,
        field_index: Some(index),
        parameter: Some(format!("{leg:?}")),
    }
}

#[tokio::test]
async fn aircraft_type_and_wake_category_use_aircraft_database() {
    let navdata = navdata().await;
    for (aircraft, wake, type_status, wake_status) in [
        ("A320", "M", ValidatorStatus::Pass, ValidatorStatus::Pass),
        ("a388", "j", ValidatorStatus::Pass, ValidatorStatus::Pass),
        ("C172", "L", ValidatorStatus::Pass, ValidatorStatus::Pass),
        ("BE20", "L", ValidatorStatus::Pass, ValidatorStatus::Pass),
        ("BE20", "M", ValidatorStatus::Pass, ValidatorStatus::Pass),
        (
            "A320",
            "H",
            ValidatorStatus::Pass,
            ValidatorStatus::Rejected,
        ),
        (
            "A320",
            "X",
            ValidatorStatus::Pass,
            ValidatorStatus::Rejected,
        ),
        (
            "A320",
            "",
            ValidatorStatus::Pass,
            ValidatorStatus::Unavailable,
        ),
        (
            "INVALID",
            "M",
            ValidatorStatus::Rejected,
            ValidatorStatus::Unavailable,
        ),
        (
            "",
            "M",
            ValidatorStatus::Unavailable,
            ValidatorStatus::Unavailable,
        ),
    ] {
        let mut flight = flight();
        flight.aircraft = aircraft.into();
        flight.wake_category = wake.into();
        let result = validate::<AircraftTypeValidator>(&navdata, &flight, vec![], None);
        assert_eq!(result.status, type_status, "{aircraft}");
        if type_status == ValidatorStatus::Rejected {
            assert_eq!(
                result.warnings[0].message_code,
                WarningMessageCode::InvalidAircraftType
            );
        }
        let result = validate::<WakeCategoryValidator>(&navdata, &flight, vec![], None);
        assert_eq!(result.status, wake_status, "{aircraft}/{wake}");
        if wake_status == ValidatorStatus::Rejected {
            assert_eq!(
                result.warnings,
                vec![WarningMessage::with_parameter(
                    WarningMessageField::WakeCategory,
                    WarningMessageCode::WakeCategoryMismatch,
                    "M"
                )]
            );
        }
    }
}

#[tokio::test]
async fn flight_rule_weather_is_suppressed_for_ifr() {
    let navdata = navdata().await;
    for (rules, status) in [
        ("I", ValidatorStatus::Suppressed),
        (" i ", ValidatorStatus::Suppressed),
        ("V", ValidatorStatus::Unavailable),
        ("", ValidatorStatus::Unavailable),
    ] {
        let mut flight = flight();
        flight.flight_rules = rules.into();
        let result = validate::<FlightRuleWeatherValidator>(&navdata, &flight, vec![], None);
        assert_eq!(result.status, status);
        assert!(result.warnings.is_empty());
    }
}

fn directional_leg(from: (f64, f64), to: (f64, f64)) -> ResolvedLeg {
    ResolvedLeg {
        from: GeoPoint::new(from.0, from.1).unwrap().into(),
        to: GeoPoint::new(to.0, to.1).unwrap().into(),
        ..leg(LegKind::Direct)
    }
}

#[tokio::test]
async fn cruise_altitude_uses_first_leg_direction_at_all_altitudes() {
    let navdata = navdata().await;
    let east = directional_leg((30., 110.), (30., 111.));
    let west = directional_leg((30., 111.), (30., 110.));
    for level in [3000, 26600, 29100, 33100, 41100, 44900, 48900] {
        let mut flight = flight();
        flight.cruising_level = level;
        assert_eq!(
            validate::<DirectionalCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![east.clone(), west.clone()],
                None
            )
            .status,
            ValidatorStatus::Pass
        );
        let result = validate::<DirectionalCruiseAltitudeValidator>(
            &navdata,
            &flight,
            vec![west.clone(), east.clone()],
            None,
        );
        assert_eq!(result.status, ValidatorStatus::Rejected);
        assert_eq!(
            result.warnings,
            vec![WarningMessage::with_parameter(
                WarningMessageField::CruisingLevel,
                WarningMessageCode::CruisingLevelMismatch,
                "standard_even"
            )]
        );
    }
    for level in [2000, 27600, 30100, 34100, 40100, 43000, 46900] {
        let mut flight = flight();
        flight.cruising_level = level;
        assert_eq!(
            validate::<DirectionalCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![west.clone()],
                None
            )
            .status,
            ValidatorStatus::Pass
        );
        assert_eq!(
            validate::<DirectionalCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![east.clone()],
                None
            )
            .status,
            ValidatorStatus::Rejected
        );
    }
    for level in [30000, 30500, 50000] {
        let mut flight = flight();
        flight.cruising_level = level;
        for leg in [&east, &west] {
            assert_eq!(
                validate::<DirectionalCruiseAltitudeValidator>(
                    &navdata,
                    &flight,
                    vec![leg.clone()],
                    None
                )
                .status,
                ValidatorStatus::Rejected
            );
        }
    }
}

#[tokio::test]
async fn cruise_altitude_prefers_route_restrictions_over_direction() {
    let navdata = navdata().await;
    let west = directional_leg((30., 111.), (30., 110.));
    let mut flight = flight();
    let mut route = preferred();
    route.cruising_level_restriction = LevelRestrictionType::StandardOdd;
    assert_eq!(
        validate::<PreferredRouteCruiseAltitudeValidator>(
            &navdata,
            &flight,
            vec![west],
            Some(&route)
        )
        .status,
        ValidatorStatus::Pass
    );
    route.allowed_altitudes = vec![35100, 37100];
    route.minimal_altitude = 35100;
    let result =
        validate::<PreferredRouteCruiseAltitudeValidator>(&navdata, &flight, vec![], Some(&route));
    assert_eq!(result.status, ValidatorStatus::Rejected);
    assert_eq!(
        result.warnings,
        vec![
            WarningMessage::with_parameter(
                WarningMessageField::CruisingLevel,
                WarningMessageCode::CruisingLevelNotAllowed,
                "35100,37100"
            ),
            WarningMessage::with_parameter(
                WarningMessageField::CruisingLevel,
                WarningMessageCode::CruisingLevelTooLow,
                "35100"
            ),
        ]
    );
    flight.cruising_level = 35100;
    assert_eq!(
        validate::<PreferredRouteCruiseAltitudeValidator>(&navdata, &flight, vec![], Some(&route))
            .status,
        ValidatorStatus::Pass
    );
    route.allowed_altitudes.clear();
    route.minimal_altitude = 0;
    for (restriction, level) in [
        (LevelRestrictionType::Standard, 30100),
        (LevelRestrictionType::FlightLevelEven, 30000),
        (LevelRestrictionType::FlightLevelOdd, 31000),
    ] {
        route.cruising_level_restriction = restriction;
        flight.cruising_level = level;
        assert_eq!(
            validate::<PreferredRouteCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![],
                Some(&route)
            )
            .status,
            ValidatorStatus::Pass
        );
    }
    route.cruising_level_restriction = LevelRestrictionType::Standard;
    flight.cruising_level = 30500;
    assert_eq!(
        validate::<PreferredRouteCruiseAltitudeValidator>(&navdata, &flight, vec![], Some(&route))
            .status,
        ValidatorStatus::Rejected
    );
}

#[tokio::test]
async fn cruise_altitude_handles_missing_direction_and_bearing_boundaries() {
    let navdata = navdata().await;
    let mut flight = flight();
    for legs in [
        vec![],
        vec![leg(LegKind::Direct)],
        vec![directional_leg((30., 110.), (30., 110.))],
        vec![ResolvedLeg {
            from: AnyFix::Unknown("UNKNOWN".into()),
            ..leg(LegKind::Direct)
        }],
    ] {
        assert_eq!(
            validate::<DirectionalCruiseAltitudeValidator>(&navdata, &flight, legs, None).status,
            ValidatorStatus::Unavailable
        );
    }
    for (from, to, level) in [
        ((0., 110.), (1., 110.), 33100),
        ((1., 110.), (0., 110.), 34100),
        ((0., 179.), (0., -179.), 33100),
        ((0., -179.), (0., 179.), 34100),
    ] {
        flight.cruising_level = level;
        assert_eq!(
            validate::<DirectionalCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![directional_leg(from, to)],
                None
            )
            .status,
            ValidatorStatus::Pass
        );
    }
    for level in [0, -1, i64::MAX] {
        flight.cruising_level = level;
        assert_eq!(
            validate::<DirectionalCruiseAltitudeValidator>(&navdata, &flight, vec![], None).status,
            ValidatorStatus::Unavailable
        );
    }
}

#[tokio::test]
async fn cruise_altitude_checks_imperial_parity_including_metric_overlaps() {
    let navdata = navdata().await;
    let mut route = preferred();
    for (restriction, accepted, rejected, parameter) in [
        (
            LevelRestrictionType::FlightLevelEven,
            vec![2000, 28000, 30000, 42000],
            vec![1000, 3000, 29100, 30500, 43000],
            "flight_level_even",
        ),
        (
            LevelRestrictionType::FlightLevelOdd,
            vec![1000, 3000, 31000, 43000],
            vec![2000, 30000, 30100, 30500, 42000],
            "flight_level_odd",
        ),
    ] {
        route.cruising_level_restriction = restriction;
        for level in accepted {
            let mut flight = flight();
            flight.cruising_level = level;
            let result = validate::<PreferredRouteCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![],
                Some(&route),
            );
            assert_eq!(
                result.status,
                ValidatorStatus::Pass,
                "{restriction:?}/{level}"
            );
            assert!(result.warnings.is_empty());
        }
        for level in rejected {
            let mut flight = flight();
            flight.cruising_level = level;
            let result = validate::<PreferredRouteCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![],
                Some(&route),
            );
            assert_eq!(
                result.status,
                ValidatorStatus::Rejected,
                "{restriction:?}/{level}"
            );
            assert_eq!(
                result.warnings,
                vec![WarningMessage::with_parameter(
                    WarningMessageField::CruisingLevel,
                    WarningMessageCode::CruisingLevelMismatch,
                    parameter
                )]
            );
        }
    }
}

#[tokio::test]
async fn altitude_checks_are_mutually_exclusive_and_skip_foreign_departure_direction() {
    let navdata = navdata().await;
    let east = directional_leg((30., 110.), (30., 111.));
    let mut flight = flight();
    let route = preferred();
    let suppressed = validate::<PreferredRouteCruiseAltitudeValidator>(
        &navdata,
        &flight,
        vec![east.clone()],
        None,
    );
    assert_eq!(suppressed.status, ValidatorStatus::Suppressed);
    assert!(suppressed.warnings.is_empty());
    let suppressed = validate::<DirectionalCruiseAltitudeValidator>(
        &navdata,
        &flight,
        vec![east.clone()],
        Some(&route),
    );
    assert_eq!(suppressed.status, ValidatorStatus::Suppressed);
    assert!(suppressed.warnings.is_empty());
    // Foreign departures suppress directional validation even without a usable level or route.
    flight.cruising_level = 0;
    for departure in ["RJAA", "VHHH", "RCTP", "ZMUB", "ZKPY"] {
        flight.departure = departure.into();
        let result =
            validate::<DirectionalCruiseAltitudeValidator>(&navdata, &flight, vec![], None);
        assert_eq!(result.status, ValidatorStatus::Suppressed, "{departure}");
        assert!(result.warnings.is_empty());
    }
    flight.departure = "RJAA".into();
    flight.cruising_level = 30500;
    assert_eq!(
        validate::<PreferredRouteCruiseAltitudeValidator>(&navdata, &flight, vec![], Some(&route))
            .status,
        ValidatorStatus::Rejected
    );
    for departure in ["", "ZZZZ", "Z", "1234"] {
        flight.departure = departure.into();
        assert_eq!(
            validate::<DirectionalCruiseAltitudeValidator>(
                &navdata,
                &flight,
                vec![east.clone()],
                None
            )
            .status,
            ValidatorStatus::Unavailable
        );
    }
    flight.departure = " zbaa ".into();
    flight.cruising_level = 33100;
    let parsed_route = ParsedRoute { legs: vec![east] };
    let results = crate::modules::flight::flight_plan::validator::validate_all(
        &ValidationContext {
            flight: &flight,
            route: &parsed_route,
            preferred_route: None,
            preferred_routes: &[],
        },
        &navdata,
    )
    .unwrap();
    assert_eq!(
        results
            .iter()
            .find(|result| result.validator_ident == PreferredRouteCruiseAltitudeValidator::IDENT)
            .unwrap()
            .status,
        ValidatorStatus::Suppressed
    );
    assert_eq!(
        results
            .iter()
            .find(|result| result.validator_ident == DirectionalCruiseAltitudeValidator::IDENT)
            .unwrap()
            .status,
        ValidatorStatus::Pass
    );
}
