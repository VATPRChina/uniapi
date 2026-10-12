#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("coordinate must be finite")]
pub struct NonFiniteCoordinateError;

fn coordinate_eq(left: f64, right: f64) -> bool {
    let left_hundredths = left * 360_000.;
    let right_hundredths = right * 360_000.;
    if !left_hundredths.is_finite() || !right_hundredths.is_finite() {
        // At this magnitude, distinct f64 values already differ by more than 0.01 arcsecond.
        return left == right;
    }
    left_hundredths.round() == right_hundredths.round()
}

struct Coordinate(f64, char, char);

impl std::fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value, positive, negative) = *self;
        // Round before splitting so seconds carry into minutes/degrees at 60.
        let hundredths = (value.abs() * 360_000.).round();
        if !hundredths.is_finite() {
            return std::fmt::Debug::fmt(&value, f);
        }
        let hemisphere = if value.is_sign_negative() {
            negative
        } else {
            positive
        };
        let degrees = (hundredths / 360_000.).floor();
        let minutes = ((hundredths % 360_000.) / 6_000.).floor();
        let seconds = (hundredths % 6_000.) / 100.;
        write!(
            f,
            "{hemisphere} {degrees:.0}°{minutes:02.0}'{seconds:05.2}\""
        )
    }
}

/// Finite decimal degrees, compared after rounding to the nearest 0.01 arcsecond.
#[derive(Clone, Copy)]
pub struct Latitude(f64);

impl Latitude {
    /// Returns an error if the coordinate is NaN or infinite.
    pub fn new(value: f64) -> Result<Self, NonFiniteCoordinateError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(NonFiniteCoordinateError)
        }
    }
}

impl std::ops::Deref for Latitude {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<f64> for Latitude {
    type Error = NonFiniteCoordinateError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl PartialEq for Latitude {
    fn eq(&self, other: &Self) -> bool {
        coordinate_eq(self.0, other.0)
    }
}

impl Eq for Latitude {}

impl std::fmt::Debug for Latitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Coordinate(self.0, 'N', 'S').fmt(f)
    }
}

/// Finite decimal degrees, compared after rounding to the nearest 0.01 arcsecond.
#[derive(Clone, Copy)]
pub struct Longitude(f64);

impl Longitude {
    /// Returns an error if the coordinate is NaN or infinite.
    pub fn new(value: f64) -> Result<Self, NonFiniteCoordinateError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(NonFiniteCoordinateError)
        }
    }
}

impl std::ops::Deref for Longitude {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<f64> for Longitude {
    type Error = NonFiniteCoordinateError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl PartialEq for Longitude {
    fn eq(&self, other: &Self) -> bool {
        coordinate_eq(self.0, other.0)
    }
}

impl Eq for Longitude {}

impl std::fmt::Debug for Longitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Coordinate(self.0, 'E', 'W').fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Latitude, Longitude};

    #[test]
    fn construction_rejects_non_finite_values() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(Latitude::new(value).is_err());
            assert!(Longitude::new(value).is_err());
            assert!(Latitude::try_from(value).is_err());
            assert!(Longitude::try_from(value).is_err());
        }
        for value in [0., -0., 30., -180., f64::MIN, f64::MAX] {
            assert_eq!(*Latitude::new(value).unwrap(), value);
            assert_eq!(*Longitude::try_from(value).unwrap(), value);
        }
    }

    #[test]
    fn equality_rounds_to_hundredths_of_an_arcsecond() {
        for degrees in [0., 30., -30., 90., -90., 180., -180.] {
            let same = degrees + 0.004 / 3600.;
            let next = degrees + 0.006 / 3600.;
            assert_eq!(
                Latitude::new(degrees).unwrap(),
                Latitude::new(same).unwrap()
            );
            assert_eq!(
                Longitude::new(degrees).unwrap(),
                Longitude::new(same).unwrap()
            );
            assert_ne!(
                Latitude::new(degrees).unwrap(),
                Latitude::new(next).unwrap()
            );
            assert_ne!(
                Longitude::new(degrees).unwrap(),
                Longitude::new(next).unwrap()
            );
        }
        assert_ne!(
            Latitude::new(0.004 / 3600.).unwrap(),
            Latitude::new(0.006 / 3600.).unwrap()
        );
        assert_ne!(
            Longitude::new(-0.004 / 3600.).unwrap(),
            Longitude::new(-0.006 / 3600.).unwrap()
        );
    }

    #[test]
    fn equality_is_an_equivalence_relation() {
        fn require_eq<T: Eq>() {}
        require_eq::<Latitude>();
        require_eq::<Longitude>();
        let values = [0.001 / 3600., 0.002 / 3600., 0.004 / 3600.];
        let [a, b, c] = values.map(|value| Latitude::new(value).unwrap());
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a, c);
        let [a, b, c] = values.map(|value| Longitude::new(value).unwrap());
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a, c);
        for value in [0., -0., f64::MIN, f64::MAX] {
            assert_eq!(Latitude::new(value).unwrap(), Latitude::new(value).unwrap());
            assert_eq!(
                Longitude::new(value).unwrap(),
                Longitude::new(value).unwrap()
            );
        }
        assert_eq!(Latitude::new(-0.).unwrap(), Latitude::new(0.).unwrap());
        assert_eq!(Longitude::new(-0.).unwrap(), Longitude::new(0.).unwrap());
        assert_ne!(
            Latitude::new(f64::MAX).unwrap(),
            Latitude::new(f64::MAX / 2.).unwrap()
        );
        assert_ne!(
            Longitude::new(f64::MIN).unwrap(),
            Longitude::new(f64::MIN / 2.).unwrap()
        );
    }
}
