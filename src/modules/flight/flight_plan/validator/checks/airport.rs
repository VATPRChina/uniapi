use super::CheckResult;
use super::{ValidationContext, Validator, WarningMessageField};
use crate::modules::flight::flight_plan::validator::{WarningMessage, WarningMessageCode};
use crate::modules::flight::models::ValidatorStatus;
use crate::modules::navdata::aircraft_limits::{AircraftLimits, WingspanMaximum};
use crate::modules::navdata::models::AnyFix;
use crate::modules::navdata::service::NavdataService;

/// Departure airport must permit the aircraft type.
pub struct DepartureAircraftTypeValidator;

impl Validator for DepartureAircraftTypeValidator {
    const IDENT: &'static str = "departure-aircraft-type";
    const FIELD: WarningMessageField = WarningMessageField::Departure;

    fn validate(context: &ValidationContext<'_>, navdata: &NavdataService) -> CheckResult {
        validate_wingspan::<Self>(
            &context.flight.departure,
            &context.flight.aircraft,
            navdata.aircraft_limits(),
        )
    }
}

/// Departure must be a valid airport.
pub struct DepartureAirportValidator;

impl Validator for DepartureAirportValidator {
    const IDENT: &'static str = "departure-airport";
    const FIELD: WarningMessageField = WarningMessageField::Departure;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        validate_airport::<Self>(
            &context.flight.departure,
            context.route.legs.first().map(|leg| &leg.from),
        )
    }
}

/// Arrival airport must permit the aircraft type.
pub struct ArrivalAircraftTypeValidator;

impl Validator for ArrivalAircraftTypeValidator {
    const IDENT: &'static str = "arrival-aircraft-type";
    const FIELD: WarningMessageField = WarningMessageField::Arrival;

    fn validate(context: &ValidationContext<'_>, navdata: &NavdataService) -> CheckResult {
        validate_wingspan::<Self>(
            &context.flight.arrival,
            &context.flight.aircraft,
            navdata.aircraft_limits(),
        )
    }
}

/// Arrival must be a valid airport.
pub struct ArrivalAirportValidator;

impl Validator for ArrivalAirportValidator {
    const IDENT: &'static str = "arrival-airport";
    const FIELD: WarningMessageField = WarningMessageField::Arrival;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        validate_airport::<Self>(
            &context.flight.arrival,
            context.route.legs.last().map(|leg| &leg.to),
        )
    }
}

fn validate_wingspan<V: Validator>(
    airport: &str,
    aircraft: &str,
    data: &AircraftLimits,
) -> CheckResult {
    let airport = airport.trim().to_ascii_uppercase();
    let aircraft = aircraft.trim().to_ascii_uppercase();
    let mut result = CheckResult {
        validator_ident: V::IDENT.to_owned(),
        field: V::FIELD,
        status: ValidatorStatus::Unavailable,
        warnings: Vec::new(),
    };
    if aircraft.is_empty() {
        return result;
    }
    let Some(limit) = data.airport(&airport) else {
        return result;
    };
    // A listed exception requires a controller's manual check, not automatic approval.
    if limit.manual_check_types.contains(&aircraft) {
        result.warnings.push(WarningMessage::with_parameter(
            V::FIELD,
            WarningMessageCode::AircraftWingspanManualCheck,
            format!("{airport}: {aircraft}"),
        ));
        return result;
    }
    let (maximum, exclusive) = match &limit.maximum {
        WingspanMaximum::Unrestricted => {
            result.status = ValidatorStatus::Suppressed;
            return result;
        }
        WingspanMaximum::Metres { value, exclusive } => (*value, *exclusive),
        WingspanMaximum::Aircraft(identifier) => match data.wingspan(identifier) {
            Some(value) => (value, false),
            None => return result,
        },
        WingspanMaximum::Unknown => return result,
    };
    let Some(wingspan) = data.wingspan(&aircraft) else {
        return result;
    };
    // Annex 14 code-letter upper bounds (marked ^ in Clearance.txt) are exclusive.
    // https://www.icao.int/sites/default/files/APAC/Meetings/2021/2021%20AP-ADO-TF2/3-Working%20Papers/WP04-AI2-Amdt-15-to-A14V1.pdf
    let permitted = if exclusive {
        wingspan < maximum
    } else {
        wingspan <= maximum
    };
    result.status = if permitted {
        ValidatorStatus::Pass
    } else {
        ValidatorStatus::Rejected
    };
    if !permitted {
        result.warnings.push(WarningMessage::with_parameter(
            V::FIELD,
            WarningMessageCode::AircraftWingspanTooLarge,
            format!(
                "{airport}: {aircraft} ({wingspan} m; {} {maximum} m)",
                if exclusive { "<" } else { "≤" }
            ),
        ));
    }
    result
}

/// The route parser has already resolved these endpoints against navdata.
fn validate_airport<V: Validator>(identifier: &str, endpoint: Option<&AnyFix>) -> CheckResult {
    let identifier = identifier.trim();
    let valid_identifier = identifier.len() == 4
        && identifier
            .bytes()
            .all(|character| character.is_ascii_alphabetic());
    let status = if identifier.is_empty() || identifier == "ZZZZ" {
        ValidatorStatus::Unavailable
    } else if !valid_identifier {
        ValidatorStatus::Rejected
    } else {
        match endpoint {
            None => ValidatorStatus::Unavailable,
            Some(AnyFix::Airport(airport))
                if airport.identifier.as_str().eq_ignore_ascii_case(identifier) =>
            {
                ValidatorStatus::Pass
            }
            Some(_) => ValidatorStatus::Rejected,
        }
    };
    let warnings = if status == ValidatorStatus::Rejected {
        vec![WarningMessage::with_parameter(
            V::FIELD,
            WarningMessageCode::InvalidAirport,
            identifier,
        )]
    } else {
        Vec::new()
    };
    CheckResult {
        validator_ident: V::IDENT.to_owned(),
        field: V::FIELD,
        status,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::flight::dto::TemporaryFlightQuery;
    use crate::modules::flight::models::{Flight, ParsedRoute};

    const WINGSPANS: &str =
        "B738 35.8 C\nB77W 64.8 E\nA388 79.8 F\nB748 68.4 F\nA225 88.4 F+\nTEST 36 C\n";
    const CLEARANCE: &str = "\
ZBAA 35 - X - -
ZBAL 1395 - 3C 30 -
ZBCZ 923 - 4C 35.8 -
ZBCD 702 - 4C 36^ -
ZBHH 1084 2400 4E 65 B748*
ZBSJ 71 900 4E 65^ A225
ZREF 0 - 4C ?B738 -
ZUNK 0 - 4C ?MISSING -
ZBAD 0 - 4C NaN -
";

    #[test]
    fn compares_wingspans_at_both_endpoints_with_correct_fields_and_identifiers() {
        let data = AircraftLimits::parse(WINGSPANS, CLEARANCE);
        for (airport, aircraft, expected) in [
            ("ZBCZ", "B738", ValidatorStatus::Pass), // equality at an explicit limit
            ("ZBCZ", "B77W", ValidatorStatus::Rejected),
            ("ZBAL", "B738", ValidatorStatus::Rejected),
            ("ZBCD", "B738", ValidatorStatus::Pass),
            ("ZBCD", "TEST", ValidatorStatus::Rejected), // equality at an inferred upper bound
            ("ZREF", "B738", ValidatorStatus::Pass),
            ("ZREF", "A388", ValidatorStatus::Rejected),
            ("ZBAA", "A388", ValidatorStatus::Suppressed),
            ("ZBCZ", "UNKNOWN", ValidatorStatus::Unavailable),
            ("ZUNK", "B738", ValidatorStatus::Unavailable),
            ("ZBAD", "B738", ValidatorStatus::Unavailable),
            ("RJAA", "B738", ValidatorStatus::Unavailable),
            ("", "B738", ValidatorStatus::Unavailable),
            ("ZZZZ", "B738", ValidatorStatus::Unavailable),
            ("ZBCZ", "", ValidatorStatus::Unavailable),
            (" zbcz ", " b738 ", ValidatorStatus::Pass),
        ] {
            let departure =
                validate_wingspan::<DepartureAircraftTypeValidator>(airport, aircraft, &data);
            let arrival =
                validate_wingspan::<ArrivalAircraftTypeValidator>(airport, aircraft, &data);
            for (result, field, ident) in [
                (
                    departure,
                    WarningMessageField::Departure,
                    "departure-aircraft-type",
                ),
                (
                    arrival,
                    WarningMessageField::Arrival,
                    "arrival-aircraft-type",
                ),
            ] {
                assert_eq!(result.status, expected, "{airport}: {aircraft} ({ident})");
                assert_eq!(result.field, field);
                assert_eq!(result.validator_ident, ident);
                if expected == ValidatorStatus::Rejected {
                    assert_eq!(result.warnings.len(), 1);
                    assert_eq!(
                        result.warnings[0].message_code,
                        WarningMessageCode::AircraftWingspanTooLarge
                    );
                    assert_eq!(result.warnings[0].field, field);
                    assert!(
                        result.warnings[0]
                            .parameter
                            .as_ref()
                            .unwrap()
                            .contains(aircraft)
                    );
                } else {
                    assert!(result.warnings.is_empty());
                }
            }
        }
    }

    #[test]
    fn special_approval_aircraft_require_controller_manual_check() {
        let data = AircraftLimits::parse(WINGSPANS, CLEARANCE);
        for (airport, aircraft) in [("ZBHH", "B748"), ("ZBSJ", "A225"), (" zbhh ", " b748 ")] {
            for result in [
                validate_wingspan::<DepartureAircraftTypeValidator>(airport, aircraft, &data),
                validate_wingspan::<ArrivalAircraftTypeValidator>(airport, aircraft, &data),
            ] {
                assert_eq!(result.status, ValidatorStatus::Unavailable);
                assert_eq!(result.warnings.len(), 1);
                assert_eq!(
                    result.warnings[0].message_code,
                    WarningMessageCode::AircraftWingspanManualCheck
                );
                assert_eq!(result.warnings[0].field, result.field);
            }
        }
        // Asterisk is an annotation on the complete designator, not a wildcard.
        assert!(
            validate_wingspan::<DepartureAircraftTypeValidator>("ZBHH", "B74", &data)
                .warnings
                .is_empty()
        );
        assert_eq!(
            validate_wingspan::<DepartureAircraftTypeValidator>("ZBHH", "A388", &data).status,
            ValidatorStatus::Rejected
        );
    }

    #[test]
    fn missing_data_never_passes_a_check() {
        let data = AircraftLimits::default();
        assert_eq!(
            validate_wingspan::<DepartureAircraftTypeValidator>("ZBAA", "B738", &data).status,
            ValidatorStatus::Unavailable
        );
        assert_eq!(
            validate_wingspan::<ArrivalAircraftTypeValidator>("ZBCZ", "B738", &data).status,
            ValidatorStatus::Unavailable
        );
    }

    #[tokio::test]
    async fn validators_load_supplemental_data_and_use_their_own_filed_airports() {
        let directory =
            std::env::temp_dir().join(format!("vatprc-aircraft-limits-{}", ulid::Ulid::new()));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("Wingspan.txt"), WINGSPANS).unwrap();
        std::fs::write(directory.join("Clearance.txt"), CLEARANCE).unwrap();
        let path = directory.join("navdata.db");
        let db = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
        db.close().await;
        let navdata = NavdataService::with_preferred_routes_path(path.to_str().unwrap(), &path)
            .await
            .unwrap();
        let mut flight = Flight::from(TemporaryFlightQuery {
            departure: "ZBAL".into(),
            arrival: "ZBCZ".into(),
            aircraft: "B738".into(),
            flight_rules: String::new(),
            wake_category: String::new(),
            equipment: String::new(),
            navigation_performance: String::new(),
            transponder: String::new(),
            raw_route: String::new(),
            cruising_level: 0,
        });
        let route = ParsedRoute { legs: vec![] };
        let context = ValidationContext {
            flight: &flight,
            route: &route,
            preferred_route: None,
            preferred_routes: &[],
        };
        assert_eq!(
            DepartureAircraftTypeValidator::validate(&context, &navdata).status,
            ValidatorStatus::Rejected
        );
        assert_eq!(
            ArrivalAircraftTypeValidator::validate(&context, &navdata).status,
            ValidatorStatus::Pass
        );
        std::mem::swap(&mut flight.departure, &mut flight.arrival);
        let context = ValidationContext {
            flight: &flight,
            route: &route,
            preferred_route: None,
            preferred_routes: &[],
        };
        assert_eq!(
            DepartureAircraftTypeValidator::validate(&context, &navdata).status,
            ValidatorStatus::Pass
        );
        assert_eq!(
            ArrivalAircraftTypeValidator::validate(&context, &navdata).status,
            ValidatorStatus::Rejected
        );
        drop(navdata);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
