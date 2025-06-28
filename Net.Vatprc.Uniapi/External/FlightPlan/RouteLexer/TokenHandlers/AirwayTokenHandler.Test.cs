using Moq;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

[TestFixture]
public class AirwayTokenHandlerTest
{
    protected AirwayTokenHandler Handler { get; set; }
    protected Mock<ILexerContext> ContextMock { get; set; }
    protected ILexerContext Context => ContextMock.Object;
    protected Mock<INavdataProvider> NavdataMock { get; set; }
    protected INavdataProvider Navdata => NavdataMock.Object;

    [SetUp]
    public void SetUp()
    {
        ContextMock = new Mock<ILexerContext>();
        NavdataMock = new Mock<INavdataProvider>();
        Handler = new AirwayTokenHandler();
    }

    [Test]
    public void IsAllowed_ShouldReturnTrue_WhenLastSegmentIsVhf()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.VHF,
            Value = "PUD",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.VHF,
            Value = "SHA",
            Id = Ulid.Empty,
        });

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeTrue();
    }

    [Test]
    public void IsAllowed_ShouldReturnTrue_WhenAdjacentSegmentIsNdb()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.NDB,
            Value = "PUD",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.NDB,
            Value = "SHA",
            Id = Ulid.Empty,
        });

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeTrue();
    }

    [Test]
    public void IsAllowed_ShouldReturnTrue_WhenAdjacentSegmentIsWaypoint()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.WAYPOINT,
            Value = "TOSID",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.WAYPOINT,
            Value = "ANSUK",
            Id = Ulid.Empty,
        });

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeTrue();
    }

    [Test]
    public void IsAllowed_ShouldReturnFalse_WhenLastSegmentIsAirway()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.AIRWAY,
            Value = "A593",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.WAYPOINT,
            Value = "ANSUK",
            Id = Ulid.Empty,
        });

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeFalse();
    }

    [Test]
    public void IsAllowed_ShouldReturnFalse_WhenLastSegmentIsAirport()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.AIRPORT,
            Value = "ZSPD",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.WAYPOINT,
            Value = "ANSUK",
            Id = Ulid.Empty,
        });

        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeFalse();
    }

    [Test]
    public async Task Resolve_ShouldSetCurrentSegmentKindToAirway_WhenExistsAirwayWithFix()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.VHF,
            Value = "PUD",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "A593",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.NDB,
            Value = "SHA",
            Id = Ulid.Empty,
        });

        NavdataMock.Setup(n => n.ExistsAirwayWithFix("A593", "PUD"))
            .ReturnsAsync(true);
        NavdataMock.Setup(n => n.ExistsAirwayWithFix("A593", "SHA"))
            .ReturnsAsync(true);

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(n => n.ExistsAirwayWithFix("A593", "PUD"), Times.Once);
        NavdataMock.Verify(n => n.ExistsAirwayWithFix("A593", "SHA"), Times.Once);
        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.AIRWAY);
        Context.CurrentSegment.Value.Should().Be("A593");
        Context.CurrentSegment.Id.Should().Be(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = It.IsAny<double>(), Times.Never);
        ContextMock.VerifySet(c => c.CurrentLon = It.IsAny<double>(), Times.Never);
    }

    [Test]
    public async Task Resolve_ShouldNotChangeCurrentSegment_WhenAirwayDoesNotExistLeft()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.VHF,
            Value = "SHA",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "A593",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.WAYPOINT,
            Value = "PUD",
            Id = Ulid.Empty,
        });

        NavdataMock.Setup(n => n.ExistsAirwayWithFix("A593", "SHA")).ReturnsAsync(false);
        NavdataMock.Setup(n => n.ExistsAirwayWithFix("A593", "PUD")).ReturnsAsync(false);

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(n => n.ExistsAirwayWithFix("A593", "SHA"), Times.Once);
        NavdataMock.Verify(n => n.ExistsAirwayWithFix("A593", "PUD"), Times.Once);
        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.UNKNOWN);
        Context.CurrentSegment.Value.Should().Be("A593");
        Context.CurrentSegment.Id.Should().Be(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = It.IsAny<double>(), Times.Never);
        ContextMock.VerifySet(c => c.CurrentLon = It.IsAny<double>(), Times.Never);
    }

    [Test]
    public async Task Resolve_ShouldNotChangeCurrentSegment_WhenAirwayDoesNotExistRight()
    {
        ContextMock.SetupGet(c => c.LastSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.VHF,
            Value = "SHA",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "A593",
            Id = Ulid.Empty,
        });
        ContextMock.SetupGet(c => c.NextSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.WAYPOINT,
            Value = "PUD",
            Id = Ulid.Empty,
        });

        NavdataMock.Setup(n => n.ExistsAirwayWithFix("A593", "SHA")).ReturnsAsync(false);
        NavdataMock.Setup(n => n.ExistsAirwayWithFix("A593", "PUD")).ReturnsAsync(false);

        await Handler.Resolve(Context, Navdata);

        NavdataMock.Verify(n => n.ExistsAirwayWithFix("A593", "SHA"), Times.Once);
        NavdataMock.Verify(n => n.ExistsAirwayWithFix("A593", "PUD"), Times.Once);
        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.UNKNOWN);
        Context.CurrentSegment.Value.Should().Be("A593");
        Context.CurrentSegment.Id.Should().Be(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = It.IsAny<double>(), Times.Never);
        ContextMock.VerifySet(c => c.CurrentLon = It.IsAny<double>(), Times.Never);
    }
}
