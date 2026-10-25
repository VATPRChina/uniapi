use super::CheckResult;
use super::{ValidationContext, Validator};
use crate::modules::flight::models::ValidatorStatus;
use crate::modules::flight::models::{ValidationField, ValidationProblem, ValidatorIdentifier};
use crate::modules::navdata::aircraft_limits::{AircraftLimits, WingspanMaximum};
use crate::modules::navdata::models::AnyFix;
use crate::modules::navdata::service::NavdataService;

/// Departure airport must permit the aircraft type.
pub struct DepartureAircraftTypeValidator;

impl Validator for DepartureAircraftTypeValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::DepartureAircraftType;
    const FIELD: ValidationField = ValidationField::Departure;

    fn validate(context: &ValidationContext<'_>, navdata: &NavdataService) -> CheckResult {
        validate_wingspan::<Self>(
            &context.flight.flight_plan.departure,
            &context.flight.flight_plan.aircraft,
            navdata.aircraft_limits(),
        )
    }
}

/// Departure must be a valid airport.
pub struct DepartureAirportValidator;

impl Validator for DepartureAirportValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::DepartureAirport;
    const FIELD: ValidationField = ValidationField::Departure;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        validate_airport::<Self>(
            &context.flight.flight_plan.departure,
            context.route.legs.first().map(|leg| &leg.from),
        )
    }
}

/// Arrival airport must permit the aircraft type.
pub struct ArrivalAircraftTypeValidator;

impl Validator for ArrivalAircraftTypeValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::ArrivalAircraftType;
    const FIELD: ValidationField = ValidationField::Arrival;

    fn validate(context: &ValidationContext<'_>, navdata: &NavdataService) -> CheckResult {
        validate_wingspan::<Self>(
            &context.flight.flight_plan.arrival,
            &context.flight.flight_plan.aircraft,
            navdata.aircraft_limits(),
        )
    }
}

/// Arrival must be a valid airport.
pub struct ArrivalAirportValidator;

impl Validator for ArrivalAirportValidator {
    const IDENT: ValidatorIdentifier = ValidatorIdentifier::ArrivalAirport;
    const FIELD: ValidationField = ValidationField::Arrival;

    fn validate(context: &ValidationContext<'_>, _navdata: &NavdataService) -> CheckResult {
        validate_airport::<Self>(
            &context.flight.flight_plan.arrival,
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
        id: V::IDENT,
        field: V::FIELD,
        status: ValidatorStatus::Unavailable,
        problems: Vec::new(),
    };
    if aircraft.is_empty() {
        return result;
    }
    let Some(limit) = data.airport(&airport) else {
        return result;
    };
    // Explicit aircraft exceptions bypass the automatic wingspan restriction.
    if limit.extra_approval.contains(&aircraft) {
        result.status = ValidatorStatus::Pass;
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
        result
            .problems
            .push(ValidationProblem::AircraftWingspanTooLarge {
                maximum: format!("{} {maximum} m", if exclusive { "<" } else { "≤" }),
            });
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
    let problems = if status == ValidatorStatus::Rejected {
        vec![ValidationProblem::InvalidAirport]
    } else {
        Vec::new()
    };
    CheckResult {
        id: V::IDENT,
        field: V::FIELD,
        status,
        problems,
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
                    ValidationField::Departure,
                    ValidatorIdentifier::DepartureAircraftType,
                ),
                (
                    arrival,
                    ValidationField::Arrival,
                    ValidatorIdentifier::ArrivalAircraftType,
                ),
            ] {
                assert_eq!(result.status, expected, "{airport}: {aircraft} ({ident:?})");
                assert_eq!(result.field, field);
                assert_eq!(result.id, ident);
                if expected == ValidatorStatus::Rejected {
                    assert_eq!(result.problems.len(), 1);
                    assert!(
                        matches!(&result.problems[0], ValidationProblem::AircraftWingspanTooLarge { maximum } if maximum.ends_with(" m"))
                    );
                } else {
                    assert!(result.problems.is_empty());
                }
            }
        }
    }

    #[test]
    fn special_approval_aircraft_pass_the_automatic_wingspan_check() {
        let data = AircraftLimits::parse(WINGSPANS, CLEARANCE);
        for (airport, aircraft) in [("ZBHH", "B748"), ("ZBSJ", "A225"), (" zbhh ", " b748 ")] {
            for result in [
                validate_wingspan::<DepartureAircraftTypeValidator>(airport, aircraft, &data),
                validate_wingspan::<ArrivalAircraftTypeValidator>(airport, aircraft, &data),
            ] {
                assert_eq!(result.status, ValidatorStatus::Pass);
                assert!(result.problems.is_empty());
            }
        }
        // Asterisk is an annotation on the complete designator, not a wildcard.
        assert!(
            validate_wingspan::<DepartureAircraftTypeValidator>("ZBHH", "B74", &data)
                .problems
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
        std::mem::swap(
            &mut flight.flight_plan.departure,
            &mut flight.flight_plan.arrival,
        );
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
