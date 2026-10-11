//! Shared China metric altitude table and preferred-route level classification.

use crate::modules::navdata::models::LevelRestrictionType;

pub(crate) fn level_restriction_type(cruising_level: i32) -> LevelRestrictionType {
    if let Some(metric_level) = standard_altitude_from_flight_level(cruising_level) {
        let is_even = match metric_level {
            // Below RVSM, levels are 300 m apart and alternate every level.
            ..=8400 => (metric_level / 300) % 2 == 0,
            // In RVSM airspace, 8,900 m is the first odd level and levels
            // alternate every 300 m through 12,500 m.
            8900..=12500 => ((metric_level - 8900) / 300) % 2 != 0,
            // Above RVSM, levels are 600 m apart and alternate every level.
            _ => ((metric_level - 12500) / 600) % 2 != 0,
        };

        if is_even {
            LevelRestrictionType::StandardEven
        } else {
            LevelRestrictionType::StandardOdd
        }
    } else if cruising_level % 1000 == 0 {
        if cruising_level % 2000 == 0 {
            LevelRestrictionType::FlightLevelEven
        } else {
            LevelRestrictionType::FlightLevelOdd
        }
    } else {
        LevelRestrictionType::Standard
    }
}

pub(crate) fn level_restriction_matches(
    actual: LevelRestrictionType,
    expected: LevelRestrictionType,
) -> bool {
    match expected {
        LevelRestrictionType::StandardEven => actual == LevelRestrictionType::StandardEven,
        LevelRestrictionType::StandardOdd => actual == LevelRestrictionType::StandardOdd,
        LevelRestrictionType::Standard => matches!(
            actual,
            LevelRestrictionType::Standard
                | LevelRestrictionType::StandardEven
                | LevelRestrictionType::StandardOdd
        ),
        LevelRestrictionType::FlightLevelEven => actual == LevelRestrictionType::FlightLevelEven,
        LevelRestrictionType::FlightLevelOdd => actual == LevelRestrictionType::FlightLevelOdd,
    }
}

pub(crate) fn level_restriction_param(restriction: LevelRestrictionType) -> String {
    match restriction {
        LevelRestrictionType::StandardEven => "standard_even",
        LevelRestrictionType::StandardOdd => "standard_odd",
        LevelRestrictionType::Standard => "standard",
        LevelRestrictionType::FlightLevelEven => "flight_level_even",
        LevelRestrictionType::FlightLevelOdd => "flight_level_odd",
    }
    .to_owned()
}

pub(crate) fn standard_altitude_from_flight_level(flight_level: i32) -> Option<i32> {
    STANDARD_ALTITUDE_TO_FLIGHT_LEVEL
        .iter()
        .find_map(|(standard, flight)| (*flight == flight_level).then_some(*standard))
}

const STANDARD_ALTITUDE_TO_FLIGHT_LEVEL: &[(i32, i32)] = &[
    (300, 1000),
    (900, 3000),
    (1500, 4900),
    (2100, 6900),
    (2700, 8900),
    (3300, 10800),
    (3900, 12800),
    (4500, 14800),
    (5100, 16700),
    (5700, 18700),
    (6300, 20700),
    (6900, 22600),
    (7500, 24600),
    (8100, 26600),
    (8900, 29100),
    (9500, 31100),
    (10100, 33100),
    (10700, 35100),
    (11300, 37100),
    (11900, 39100),
    (12500, 41100),
    (13700, 44900),
    (14900, 48900),
    (600, 2000),
    (1200, 3900),
    (1800, 5900),
    (2400, 7900),
    (3000, 9800),
    (3600, 11800),
    (4200, 13800),
    (4800, 15700),
    (5400, 17700),
    (6000, 19700),
    (6600, 21700),
    (7200, 23600),
    (7800, 25600),
    (8400, 27600),
    (9200, 30100),
    (9800, 32100),
    (10400, 34100),
    (11000, 36100),
    (11600, 38100),
    (12200, 40100),
    (13100, 43000),
    (14300, 46900),
];
