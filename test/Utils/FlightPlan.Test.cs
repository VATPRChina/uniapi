using NUnit.Framework;

namespace Net.Vatprc.Uniapi.Utils;

[TestFixture]
public class FlightPlanTest
{
    [Test]
    public void TestParseAircraft()
    {
        FlightPlanUtils.ParseIcaoAircraftCode("TBM9/L-SDFGRWY/S", "PBN/O2O3D2D3S1 DOF/250131 REG/PKGNN EET/WIIF0322 SEL/ABRS CODE/00B090 OPR/GIA PER/C RMK/TCAS SIMBRIEF /V/").Should().Be(new FlightPlanUtils.Aircraft
        (
            AircraftCode: "TBM9",
            Equipment: "SDFGRWY",
            Transponder: "S",
            NavigationPerformance: "O2O3D2D3S1"
        ));
        FlightPlanUtils.ParseIcaoAircraftCode("A20N", "DOF/250130 /V/").Should().Be(new FlightPlanUtils.Aircraft
        (
            AircraftCode: "A20N",
            Equipment: string.Empty,
            Transponder: string.Empty,
            NavigationPerformance: string.Empty
        ));
        FlightPlanUtils.ParseIcaoAircraftCode("A21N/M-VGDW/C", "PBN/A1B1C1D1L1O2S2 ").Should().Be(new FlightPlanUtils.Aircraft
        (
            AircraftCode: "A21N",
            Equipment: "VGDW",
            Transponder: "C",
            NavigationPerformance: "A1B1C1D1L1O2S2"
        ));
    }
}
