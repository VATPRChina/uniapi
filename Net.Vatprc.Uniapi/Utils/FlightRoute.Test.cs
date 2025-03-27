using System.Configuration;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Npgsql;
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

    // [Test]
    // public async Task TestNormalizeRoute()
    // {
    //     var connectionString = "Host=localhost;Username=xfoxfu;Database=vatprc;Include Error Detail=true";
    //     var dataSource = new NpgsqlDataSourceBuilder(connectionString).EnableDynamicJson().Build();
    //     var db = new VATPRCContext(new DbContextOptionsBuilder<VATPRCContext>().UseSnakeCaseNamingConvention().UseNpgsql(dataSource).Options);

    //     (await FlightRoute.NormalizeRoute(db, "ZBAA", "ZGGG", "RUSD9Z RUSDO W45 IKAVO"))
    //                                              .Should().Be("RUSD9Z RUSDO W45 ADPUM W45 IPLEV W45 NOMOV W45 APEXU W45 VAGBI W45 SQ W45 URBIL W45 UBDUN W45 NUNGA W45 GU W45 LYA W45 URGIN W45 ML W45 VARDU W45 OVLAR W45 RUXIL W45 IGEDU W45 LIN W45 CD W45 XOPEK W45 IRSAS W45 NUPTI W45 VESUX W45 PUNIR W45 IKAVO STAR");
    // }
}
