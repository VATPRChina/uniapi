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

pub struct Latitude(pub f64);

impl std::fmt::Debug for Latitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Coordinate(self.0, 'N', 'S').fmt(f)
    }
}

pub struct Longitude(pub f64);

impl std::fmt::Debug for Longitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Coordinate(self.0, 'E', 'W').fmt(f)
    }
}
