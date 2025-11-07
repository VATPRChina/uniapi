namespace Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

[TestFixture]
public class CruiseAltitudeParserTests
{
    [Test]
    public void TryParse_Meters10_M_ReturnsMeters()
    {
        CruiseAltitudeParser.TryParse("M0123", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseAltitudeParser.Kind.MetricAltitude);
        res.Value.Should().Be(1230); // 123 * 10 meters
        res.Raw.Should().Be("M0123");
    }

    [Test]
    public void TryParse_FlightLevelMeters_S_ReturnsFLMeters()
    {
        CruiseAltitudeParser.TryParse("s0456", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseAltitudeParser.Kind.MetricFlightLevel);
        res.Value.Should().Be(4560);
        res.Raw.Should().Be("S0456");
    }

    [Test]
    public void TryParse_Feet100_A_ReturnsFeet()
    {
        CruiseAltitudeParser.TryParse("A123", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseAltitudeParser.Kind.ImperialAltitude);
        res.Value.Should().Be(12300); // 123 * 100 feet
        res.Raw.Should().Be("A123");
    }

    [Test]
    public void TryParse_FlightLevelFeet_F_ReturnsFLFeet()
    {
        CruiseAltitudeParser.TryParse("f045", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseAltitudeParser.Kind.ImperialFlightLevel);
        res.Value.Should().Be(4500);
        res.Raw.Should().Be("F045");
    }

    [Test]
    public void TryParse_Invalid_ReturnsFalse()
    {
        CruiseAltitudeParser.TryParse(null, out _).Should().BeFalse();
        CruiseAltitudeParser.TryParse("", out _).Should().BeFalse();
        CruiseAltitudeParser.TryParse("M123", out _).Should().BeFalse(); // too short for M
        CruiseAltitudeParser.TryParse("A12", out _).Should().BeFalse(); // too short for A/F
        CruiseAltitudeParser.TryParse("X1234", out _).Should().BeFalse(); // invalid prefix
        CruiseAltitudeParser.TryParse("F0A5", out _).Should().BeFalse(); // non-digit
    }
}
