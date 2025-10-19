namespace Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

[TestFixture]
public class CruiseSpeedParserTests
{
    [Test]
    public void TryParse_Kph_ReturnsKph()
    {
        CruiseSpeedParser.TryParse("K0750", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseSpeedParser.Kind.Kph);
        res.Value.Should().Be(750);
        res.Raw.Should().Be("K0750");
    }

    [Test]
    public void TryParse_Kts_ReturnsKts()
    {
        CruiseSpeedParser.TryParse("n0480", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseSpeedParser.Kind.Kts);
        res.Value.Should().Be(480);
        res.Raw.Should().Be("N0480");
    }

    [Test]
    public void TryParse_Mach_ReturnsMach()
    {
        CruiseSpeedParser.TryParse("M080", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseSpeedParser.Kind.Mach);
        res.Value.Should().BeApproximately(0.80, 0.0001);
        res.Raw.Should().Be("M080");
    }

    [Test]
    public void TryParse_Invalid_ReturnsFalse()
    {
        CruiseSpeedParser.TryParse(null, out _).Should().BeFalse();
        CruiseSpeedParser.TryParse("", out _).Should().BeFalse();
        CruiseSpeedParser.TryParse("K750", out _).Should().BeFalse(); // too short
        CruiseSpeedParser.TryParse("X1234", out _).Should().BeFalse(); // invalid prefix
        CruiseSpeedParser.TryParse("M08A", out _).Should().BeFalse(); // non-digit
    }

    [Test]
    public void TryParse_VFR_ReturnsVFR()
    {
        CruiseSpeedParser.TryParse("VFR", out var res).Should().BeTrue();
        res.Should().NotBeNull();
        res!.Kind.Should().Be(CruiseSpeedParser.Kind.VFR);
        res.Value.Should().Be(0);
        res.Raw.Should().Be("VFR");
    }

    [Test]
    public void TryParse_AllowExtraChars()
    {
        CruiseSpeedParser.TryParse("K0750", out var _).Should().BeTrue();
        CruiseSpeedParser.TryParse("n0480", out var _).Should().BeTrue();
        CruiseSpeedParser.TryParse("M080", out var _).Should().BeTrue();
    }
}
