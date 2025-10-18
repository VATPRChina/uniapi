using Moq;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Lexing.TokenHandlers;

[TestFixture]
public class StarTokenHandlerTest
{
    protected StarTokenHandler Handler { get; set; }
    protected Mock<ILexerContext> ContextMock { get; set; }
    protected ILexerContext Context => ContextMock.Object;
    protected Mock<INavdataProvider> NavdataMock { get; set; }
    protected INavdataProvider Navdata => NavdataMock.Object;

    [SetUp]
    public void SetUp()
    {
        ContextMock = new Mock<ILexerContext>();
        NavdataMock = new Mock<INavdataProvider>();
        Handler = new StarTokenHandler();
    }

    [Test]
    public void IsAllowed_ShouldReturnTrue_WhenLastSegmentIsAirport()
    {
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.AIRPORT,
            Value = "EGLL",
            Id = Ulid.Empty,
        });

        var result = Handler.IsAllowed(Context, Navdata);

        result.Should().BeTrue();
    }

    [Test]
    public void IsAllowed_ShouldReturnFalse_WhenLastSegmentIsNotAirport()
    {
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "TOSID",
            Id = Ulid.Empty,
        });

        var result = Handler.IsAllowed(Context, Navdata);

        result.Should().BeFalse();
    }

    [Test]
    public async Task Resolve_ShouldReturnStar_WhenFound()
    {
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.AIRPORT,
            Value = "ZYTX",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "TOS71A",
            Id = Ulid.Empty,
        });

        var procedure = new Models.Navdata.Procedure
        {
            AirportId = Ulid.NewUlid(),
            Identifier = "TOS71A",
            SubsectionCode = 'E',
        };
        NavdataMock.Setup(n => n.FindStar("TOS71A", "ZYTX")).ReturnsAsync(procedure);

        await Handler.Resolve(Context, Navdata);

        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.STAR);
        Context.CurrentSegment.Value.Should().Be("TOS71A");
        Context.CurrentSegment.Id.Should().Be(procedure.Id);
        ContextMock.VerifySet(c => c.CurrentLat = It.IsAny<double>(), Times.Never);
        ContextMock.VerifySet(c => c.CurrentLon = It.IsAny<double>(), Times.Never);
    }

    [Test]
    public async Task Resolve_ShouldNotChangeSegment_WhenStarNotFound()
    {
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.AIRPORT,
            Value = "ZYTX",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "TOS71D",
            Id = Ulid.Empty,
        });

        NavdataMock.Setup(n => n.FindStar("TOS71D", "ZYTX")).ReturnsAsync(() => null);

        await Handler.Resolve(Context, Navdata);

        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.UNKNOWN);
        Context.CurrentSegment.Value.Should().Be("TOS71D");
        Context.CurrentSegment.Id.Should().Be(Ulid.Empty);
    }

    [Test]
    public async Task Resolve_ShouldFindStarInAirport()
    {
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.AIRPORT,
            Value = "ZYTX",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "TOS71D",
            Id = Ulid.Empty,
        });

        await Handler.Resolve(Context, Navdata);
        NavdataMock.Verify(n => n.FindStar(It.IsAny<string>(), "ZYTX"), Times.Once,
            "STAR should be searched with the correct airport identifier.");
    }
}
