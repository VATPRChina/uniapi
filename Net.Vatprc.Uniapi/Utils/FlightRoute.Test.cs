using NUnit.Framework;

namespace Net.Vatprc.Uniapi.Utils;

[TestFixture]
public class FlightRouteTest
{
    [Test]
    public void TestSimplifyRoute()
    {
        FlightRoute.SimplifyRoute("POMOK G330 UNTAN/K0907S1310 G330 PIMOL A593 DALIM W157 BEKDO/K0900S1250 W157 AVBOX AVBOX6J")
            .Should().Be("POMOK G330 PIMOL A593 DALIM W157 AVBOX AVBOX6J");
    }
    // [Test]
    // public void TestIsSpeedOrAltitude()
    // {
    //     //--- Altitude ---//
    //     // F = flight level: followed by 3 digits expressed in hundreds of feet above transition altitude.
    //     FlightRoute.IsSpeedOrAltitude("F130").Should().BeTrue();
    //     // A = altitude: followed by 3 digits expressed in hundreds of feet below transition altitude.
    //     FlightRoute.IsSpeedOrAltitude("A025").Should().BeTrue();
    //     // S = standard metric level: followed by 4 digits expressed in tens of meters above transition altitude.
    //     FlightRoute.IsSpeedOrAltitude("S1130").Should().BeTrue();
    //     // M = metric altitude: followed by 4 digits expressed in tens of meters below transition altitude.
    //     FlightRoute.IsSpeedOrAltitude("M1130").Should().BeTrue();
    //     // VFR = VFR level: it is used when no specific VFR altitude chosen.
    //     FlightRoute.IsSpeedOrAltitude("VFR").Should().BeTrue();

    //     //--- Speed ---//
    //     // N = Knots: N followed by 4 digits which will be the speed in knots.
    //     FlightRoute.IsSpeedOrAltitude("N0220").Should().BeTrue();
    //     // M = Mach: M followed by 3 digits which will be the mach number without the dot character.
    //     FlightRoute.IsSpeedOrAltitude("M079").Should().BeTrue();
    //     // K = km/h : K followed by 4 digits which will be the speed in kilometre per hour.
    //     FlightRoute.IsSpeedOrAltitude("K0350").Should().BeTrue();

    //     //--- Mixed ---//
    //     FlightRoute.IsSpeedOrAltitude("N0250F180").Should().BeTrue();
    //     FlightRoute.IsSpeedOrAltitude("K0875S1100").Should().BeTrue();
    //     FlightRoute.IsSpeedOrAltitude("K0875S1100").Should().BeTrue();
    //     FlightRoute.IsSpeedOrAltitude("K0891S1100").Should().BeTrue();
    //     FlightRoute.IsSpeedOrAltitude("N0482F320").Should().BeTrue();
    //     FlightRoute.IsSpeedOrAltitude("K0885S1190PLUS").Should().BeTrue();
    // }
}
