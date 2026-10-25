use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Supplemental data distributed as Wingspan.txt and Clearance.txt.
#[derive(Debug, Default)]
pub(crate) struct AircraftLimits {
    wingspans: HashMap<String, f64>,
    airports: HashMap<String, AirportWingspanLimit>,
}

#[derive(Debug)]
pub(crate) struct AirportWingspanLimit {
    pub maximum: WingspanMaximum,
    pub extra_approval: HashSet<String>,
}

#[derive(Debug)]
pub(crate) enum WingspanMaximum {
    Unrestricted,
    Metres {
        value: f64,
        exclusive: bool,
    },
    /// Clearance.txt can retain an unconverted aircraft designator as ?TYPE.
    Aircraft(String),
    Unknown,
}

impl AircraftLimits {
    pub fn load(directory: &Path) -> Self {
        Self::parse(
            &read_optional(&directory.join("Wingspan.txt")),
            &read_optional(&directory.join("Clearance.txt")),
        )
    }

    pub(crate) fn parse(wingspans: &str, clearance: &str) -> Self {
        let mut data = Self::default();
        for (line_number, fields) in rows(wingspans) {
            let Some((identifier, value)) = fields.first().zip(fields.get(1)) else {
                tracing::warn!(line = line_number, "invalid Wingspan.txt row");
                continue;
            };
            let Some(value) = positive_metres(value) else {
                tracing::warn!(line = line_number, "invalid aircraft wingspan");
                continue;
            };
            data.wingspans
                .insert(identifier.to_ascii_uppercase(), value);
        }
        for (line_number, fields) in rows(clearance) {
            if fields.len() < 6 {
                tracing::warn!(line = line_number, "invalid Clearance.txt row");
                continue;
            }
            let maximum = match fields[4] {
                "-" => WingspanMaximum::Unrestricted,
                value if value.starts_with('?') => {
                    WingspanMaximum::Aircraft(value[1..].to_ascii_uppercase())
                }
                value => {
                    let exclusive = value.ends_with('^');
                    match positive_metres(value.trim_end_matches('^')) {
                        Some(value) => WingspanMaximum::Metres { value, exclusive },
                        None => {
                            tracing::warn!(line = line_number, "invalid airport wingspan limit");
                            WingspanMaximum::Unknown
                        }
                    }
                }
            };
            let manual_check_types = fields[5]
                .split('/')
                .filter_map(|identifier| {
                    let identifier = identifier.trim_end_matches('*');
                    (!identifier.is_empty() && identifier != "-")
                        .then(|| identifier.to_ascii_uppercase())
                })
                .collect();
            data.airports.insert(
                fields[0].to_ascii_uppercase(),
                AirportWingspanLimit {
                    maximum,
                    extra_approval: manual_check_types,
                },
            );
        }
        data
    }

    pub fn wingspan(&self, aircraft: &str) -> Option<f64> {
        self.wingspans.get(aircraft).copied()
    }

    pub fn airport(&self, identifier: &str) -> Option<&AirportWingspanLimit> {
        self.airports.get(identifier)
    }
}

fn positive_metres(value: &str) -> Option<f64> {
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.)
}

fn rows(text: &str) -> impl Iterator<Item = (usize, Vec<&str>)> {
    text.lines().enumerate().filter_map(|(index, line)| {
        let line = line.split('#').next().unwrap_or_default().trim();
        (!line.is_empty()).then(|| (index + 1, line.split_whitespace().collect()))
    })
}

fn read_optional(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| {
        tracing::warn!(path = %path.display(), %error, "supplemental aircraft limit data is unavailable");
        String::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_derived_unrestricted_and_reference_limits() {
        let data = AircraftLimits::parse(
            "# aircraft\nB738\t35.8\tC\nA225 88.4 F+ # comment\n",
            "\
ZAAA 0 - 4C 35.8 -
ZBBB 0 - 4C 36^ A225/B748*
ZCCC 0 - X - -
ZDDD 0 - 4C ?B738 -",
        );
        assert_eq!(data.wingspan("B738"), Some(35.8));
        assert_eq!(data.wingspan("A225"), Some(88.4));
        assert!(matches!(
            data.airport("ZAAA").unwrap().maximum,
            WingspanMaximum::Metres {
                value: 35.8,
                exclusive: false
            }
        ));
        let derived = data.airport("ZBBB").unwrap();
        assert!(matches!(
            derived.maximum,
            WingspanMaximum::Metres {
                value: 36.,
                exclusive: true
            }
        ));
        assert!(derived.extra_approval.contains("B748"));
        assert!(derived.extra_approval.contains("A225"));
        assert!(!derived.extra_approval.contains("B74"));
        assert!(matches!(
            data.airport("ZCCC").unwrap().maximum,
            WingspanMaximum::Unrestricted
        ));
        assert!(
            matches!(&data.airport("ZDDD").unwrap().maximum, WingspanMaximum::Aircraft(identifier) if identifier == "B738")
        );
    }

    #[test]
    fn malformed_and_nonfinite_values_do_not_become_permissive_limits() {
        let data = AircraftLimits::parse(
            "BAD NaN\nNEG -1\nINF inf\nSHORT\n",
            "ZAAA 0 - 4C NaN -\nZBBB 0 - 4C -1 -\nSHORT\n",
        );
        assert!(data.wingspan("BAD").is_none());
        assert!(data.wingspan("NEG").is_none());
        assert!(data.wingspan("INF").is_none());
        assert!(matches!(
            data.airport("ZAAA").unwrap().maximum,
            WingspanMaximum::Unknown
        ));
        assert!(matches!(
            data.airport("ZBBB").unwrap().maximum,
            WingspanMaximum::Unknown
        ));
        assert!(data.airport("SHORT").is_none());
    }

    #[test]
    fn missing_files_leave_limits_unavailable() {
        let data = AircraftLimits::load(&std::env::temp_dir().join(ulid::Ulid::new().to_string()));
        assert!(data.wingspan("B738").is_none());
        assert!(data.airport("ZBAA").is_none());
    }
}
