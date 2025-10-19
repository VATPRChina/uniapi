using Moq;
using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

[TestFixture]
public class Geo11CharTokenHandlerTest
{
    protected Geo11CharTokenHandler Handler { get; set; }
    protected Mock<ILexerContext> ContextMock { get; set; }
    protected ILexerContext Context => ContextMock.Object;
    protected Mock<INavdataProvider> NavdataMock { get; set; }
    protected INavdataProvider Navdata => NavdataMock.Object;

    [SetUp]
    public void SetUp()
    {
        ContextMock = new Mock<ILexerContext>();
        NavdataMock = new Mock<INavdataProvider>();
        Handler = new Geo11CharTokenHandler();
    }

    [Test]
    [TestCase("5100N00000E")]
    [TestCase("5100S00000W")]
    [TestCase("5100N00000E")]
    [TestCase("5100S00000W")]
    public void IsAllowed_ShouldReturnTrue_IfInFormat(string value)
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = value,
            Id = Ulid.Empty,
        });
        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeTrue();
    }

    [Test]
    public void IsAllowed_ShouldReturnFalse_IfTooLong()
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "5100N00100W1",
            Id = Ulid.Empty,
        });
        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeFalse();
    }

    [Test]
    public void IsAllowed_ShouldReturnFalse_IfTooShort()
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = "5100N00100",
            Id = Ulid.Empty,
        });
        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeFalse();
    }

    [Test]
    [TestCase("5100N00099X")]
    [TestCase("5100S00099X")]
    [TestCase("5100X00099E")]
    [TestCase("5100X00099W")]
    public void IsAllowed_ShouldReturnFalse_IfInvalidChar(string value)
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = value,
            Id = Ulid.Empty,
        });
        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeFalse();
    }

    [Test]
    [TestCase("3830N05415E", 38.50, 54.25)]
    [TestCase("3830S05415W", -38.50, -54.25)]
    [TestCase("3804N16725W", 38.06666666666667, -167.41666666666666)]
    public async Task Resolve_ShouldParseCoord(string value, double lat, double lon)
    {
        ContextMock.SetupGet(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = value,
            Id = Ulid.Empty,
        });

        await Handler.Resolve(Context, Navdata);

        Context.CurrentSegment.Kind.Should().Be(RouteTokenKind.GEO_COORD);
        Context.CurrentSegment.Value.Should().Be(value);
        Context.CurrentSegment.Id.Should().Be(Ulid.Empty);
        ContextMock.VerifySet(c => c.CurrentLat = lat, Times.Once);
        ContextMock.VerifySet(c => c.CurrentLon = lon, Times.Once);
    }
}
