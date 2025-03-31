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
        FlightRoute.SimplifyRoute("BEKOL1Y/07C BEKOL A461 SHL W22 TEPID W24 OSNOV G586 QP B330 ELKAL W179 IGNAK/K0926S1070 W179 WFX/K0926S1040 W179 OMBON W195 VIDEK/K0907S1070 W195 LAXAL/K0907S1040 W195 IDOPA W197 ANDIM/K0907S1100 B215 IBANO G470 AKLAS W192 FKG A368 SARIN/K0898F360 N161 MADEV/K0898F380 N161 GASBI/N0485F380 N161 LEYLA N644 AGISO/N0487F400 N644 TETRO M10 SARPI UM10 GOKPA UL746 ODERO DCT OPT DCT DEGET DCT BEGLA DCT DEXIT DCT INBED DCT BOMBI DCT DENUT L610 LAM UN57 WELIN T420 ELVOS")
            .Should().Be("BEKOL1Y BEKOL A461 SHL W22 TEPID W24 OSNOV G586 QP B330 ELKAL W179 OMBON W195 IDOPA W197 ANDIM B215 IBANO G470 AKLAS W192 FKG A368 SARIN N161 LEYLA N644 TETRO M10 SARPI UM10 GOKPA UL746 ODERO DCT OPT DCT DEGET DCT BEGLA DCT DEXIT DCT INBED DCT BOMBI DCT DENUT L610 LAM UN57 WELIN T420 ELVOS");
        // BEKOL1Y BEKOL A461 SHL W22 TEPID W24 OSNOV G586 QP B330 ELKAL W179 OMBON W195 IDOPA W197 ANDIM B215 IBANO G470 AKLAS W192 FKG A368 SARIN N161 LEYLA N644 TETRO M10 SARPI UM10 GOKPA UL746 ODERO DCT DENUT L610 LAM UN57 WELIN T420
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
