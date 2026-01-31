using Arinc424.Ground;
using Moq;
using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

[TestFixture]
public class AirportTokenHandlerTest
{
    protected AirportTokenHandler Handler { get; set; }
    protected Mock<ILexerContext> ContextMock { get; set; }
    protected ILexerContext Context => ContextMock.Object;
    protected Mock<INavdataProvider> NavdataMock { get; set; }
    protected INavdataProvider Navdata => NavdataMock.Object;

    [SetUp]
    public void SetUp()
    {
        ContextMock = new Mock<ILexerContext>();
        NavdataMock = new Mock<INavdataProvider>();
        Handler = new AirportTokenHandler();
    }

    [Test]
    public void ShouldBeAllowedOnFirstSegment()
    {
        ContextMock.SetupGet(c => c.CurrentSegmentIndex).Returns(0);
        ContextMock.SetupGet(c => c.SegmentCount).Returns(10);

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeTrue();
    }

    [Test]
    public void ShouldBeAllowedOnLastSegment()
    {
        ContextMock.SetupGet(c => c.CurrentSegmentIndex).Returns(9);
        ContextMock.SetupGet(c => c.SegmentCount).Returns(10);

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeTrue();
    }

    [Test]
    public void ShouldNotBeAllowedOnOtherSegments()
    {
        ContextMock.SetupGet(c => c.CurrentSegmentIndex).Returns(5);
        ContextMock.SetupGet(c => c.SegmentCount).Returns(10);

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeFalse();
    }

    [Test]
    public async Task ShouldResolveAirportToken()
    {
        ContextMock.SetupGet(c => c.CurrentSegmentIndex).Returns(0);
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "EGLL",
            Id = string.Empty,
            Geo = null,
        });

        var airport = new Airport
        {
            Identifier = "EGLL",
            Coordinates = new(51.4775, -0.461389),
            Elevation = 25,
        };
        NavdataMock.Setup(p => p.FindAirport("EGLL")).Returns(airport);

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(p => p.FindAirport("EGLL"), Times.Once);

        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.AIRPORT);
        Context.CurrentSegment.Value.Should().Be("EGLL");
        Context.CurrentSegment.Id.Should().NotBe(string.Empty);
        Context.CurrentSegment.Geo.Should().Be(airport);
        ContextMock.VerifySet(c => c.CurrentLat = 51.4775, Times.Once);
        ContextMock.VerifySet(c => c.CurrentLon = -0.461389, Times.Once);
    }

    [Test]
    public async Task ShouldKeepNotFound()
    {
        ContextMock.SetupGet(c => c.CurrentSegmentIndex).Returns(0);
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "EGLL",
            Id = string.Empty,
            Geo = null,
        });

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(p => p.FindAirport("EGLL"), Times.Once);

        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.UNKNOWN);
        Context.CurrentSegment.Value.Should().Be("EGLL");
        Context.CurrentSegment.Id.Should().Be(string.Empty);
        Context.CurrentSegment.Geo.Should().Be(null);
        ContextMock.VerifySet(c => c.CurrentLat = It.IsAny<double>(), Times.Never);
        ContextMock.VerifySet(c => c.CurrentLon = It.IsAny<double>(), Times.Never);
    }
}
