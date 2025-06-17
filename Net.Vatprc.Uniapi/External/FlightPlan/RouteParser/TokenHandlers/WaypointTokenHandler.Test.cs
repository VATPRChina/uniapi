using Moq;
using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser.TokenHandlers;

[TestFixture]
public class WaypointTokenHandlerTest
{
    protected WaypointTokenHandler Handler { get; set; }
    protected Mock<IParseContext> ContextMock { get; set; }
    protected IParseContext Context => ContextMock.Object;
    protected Mock<INavdataProvider> NavdataMock { get; set; }
    protected INavdataProvider Navdata => NavdataMock.Object;

    [SetUp]
    public void SetUp()
    {
        ContextMock = new Mock<IParseContext>();
        NavdataMock = new Mock<INavdataProvider>();
        Handler = new WaypointTokenHandler();
    }

    [Test]
    public void IsAllowed_ShouldAlwaysReturnTrue()
    {
        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeTrue();
    }

    [Test]
    public async Task Resolve_ShouldFindVhfVor()
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "PUD",
            Id = Ulid.Empty,
        });
        NavdataMock.Setup(p => p.FindVhfNavaid("PUD", It.IsAny<double>(), It.IsAny<double>())).ReturnsAsync(new VhfNavaid
        {
            VorIdentifier = "PUD",
            VorLatitude = 51.4775,
            VorLongitude = -0.461389,
        });

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(n => n.FindVhfNavaid("PUD", It.IsAny<double>(), It.IsAny<double>()), Times.Once,
            "VHF database should be searched with the correct identifier.");
        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.VHF);
        Context.CurrentSegment.Value.Should().Be("PUD");
        Context.CurrentSegment.Id.Should().NotBe(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = 51.4775, Times.Once);
        ContextMock.VerifySet(c => c.CurrentLon = -0.461389, Times.Once);
    }

    [Test]
    public async Task Resolve_ShouldFindVhfDme()
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "PUD",
            Id = Ulid.Empty,
        });
        NavdataMock.Setup(p => p.FindVhfNavaid("PUD", It.IsAny<double>(), It.IsAny<double>())).ReturnsAsync(new VhfNavaid
        {
            DmeIdentifier = "PUD",
            DmeLatitude = 51.4775,
            DmeLongitude = -0.461389,
        });

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(n => n.FindVhfNavaid("PUD", It.IsAny<double>(), It.IsAny<double>()), Times.Once,
            "VHF database should be searched with the correct identifier.");
        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.VHF);
        Context.CurrentSegment.Value.Should().Be("PUD");
        Context.CurrentSegment.Id.Should().NotBe(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = 51.4775, Times.Once);
        ContextMock.VerifySet(c => c.CurrentLon = -0.461389, Times.Once);
    }

    [Test]
    public async Task Resolve_ShouldFindNdb()
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "SHA",
            Id = Ulid.Empty,
        });
        NavdataMock.Setup(p => p.FindNdbNavaid("SHA", It.IsAny<double>(), It.IsAny<double>())).ReturnsAsync(new NdbNavaid
        {
            Identifier = "SHA",
            Latitude = 51.4775,
            Longitude = -0.461389,
        });

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(n => n.FindVhfNavaid("SHA", It.IsAny<double>(), It.IsAny<double>()), Times.Once,
            "VHF database should be searched with the correct identifier.");
        NavdataMock.Verify(n => n.FindNdbNavaid("SHA", It.IsAny<double>(), It.IsAny<double>()), Times.Once,
            "NDB database should be searched with the correct identifier.");
        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.NDB);
        Context.CurrentSegment.Value.Should().Be("SHA");
        Context.CurrentSegment.Id.Should().NotBe(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = 51.4775, Times.Once);
        ContextMock.VerifySet(c => c.CurrentLon = -0.461389, Times.Once);
    }

    [Test]
    public async Task Resolve_ShouldFindWaypoint()
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "TOSID",
            Id = Ulid.Empty,
        });
        NavdataMock.Setup(p => p.FindWaypoint("TOSID", It.IsAny<double>(), It.IsAny<double>())).ReturnsAsync(new Waypoint
        {
            Identifier = "TOSID",
            Latitude = 51.4775,
            Longitude = -0.461389,
        });

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(n => n.FindVhfNavaid("TOSID", It.IsAny<double>(), It.IsAny<double>()), Times.Once,
            "VHF database should be searched with the correct identifier.");
        NavdataMock.Verify(n => n.FindNdbNavaid("TOSID", It.IsAny<double>(), It.IsAny<double>()), Times.Once,
            "NDB database should be searched with the correct identifier.");
        NavdataMock.Verify(n => n.FindWaypoint("TOSID", It.IsAny<double>(), It.IsAny<double>()), Times.Once,
            "Waypoint database should be searched with the correct identifier.");
        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.WAYPOINT);
        Context.CurrentSegment.Value.Should().Be("TOSID");
        Context.CurrentSegment.Id.Should().NotBe(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = 51.4775, Times.Once);
        ContextMock.VerifySet(c => c.CurrentLon = -0.461389, Times.Once);
    }
}
