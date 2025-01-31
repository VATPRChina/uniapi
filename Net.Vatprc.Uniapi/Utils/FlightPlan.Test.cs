using NUnit.Framework;

namespace Net.Vatprc.Uniapi.Utils;

[TestFixture]
public class FlightPlanTest
{
    [Test]
    public void TestParseAircraft()
    {
        FlightPlan.ParseIcaoAircraftCode("TBM9/L-SDFGRWY/S", "PBN/O2O3D2D3S1 DOF/250131 REG/PKGNN EET/WIIF0322 SEL/ABRS CODE/00B090 OPR/GIA PER/C RMK/TCAS SIMBRIEF /V/").Should().Be(new FlightPlan.Aircraft
        (
            AircraftCode: "TBM9",
            Equipment: "SDFGRWY",
            Transponder: "S",
            NavigationPerformance: "O2O3D2D3S1"
        ));
        FlightPlan.ParseIcaoAircraftCode("A20N", "DOF/250130 /V/").Should().Be(new FlightPlan.Aircraft
        (
            AircraftCode: "A20N",
            Equipment: string.Empty,
            Transponder: string.Empty,
            NavigationPerformance: string.Empty
        ));
    }
}
