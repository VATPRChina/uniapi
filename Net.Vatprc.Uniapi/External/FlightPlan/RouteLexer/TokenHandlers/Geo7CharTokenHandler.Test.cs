using Moq;
using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

[TestFixture]
public class Geo7CharTokenHandlerTest
{
    protected Geo7CharTokenHandler Handler { get; set; }
    protected Mock<ILexerContext> ContextMock { get; set; }
    protected ILexerContext Context => ContextMock.Object;
    protected Mock<INavdataProvider> NavdataMock { get; set; }
    protected INavdataProvider Navdata => NavdataMock.Object;

    [SetUp]
    public void SetUp()
    {
        ContextMock = new Mock<ILexerContext>();
        NavdataMock = new Mock<INavdataProvider>();
        Handler = new Geo7CharTokenHandler();
    }

    [Test]
    [TestCase("51N000E")]
    [TestCase("51S000W")]
    [TestCase("51N000E")]
    [TestCase("51S000W")]
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
            Value = "51N001W1",
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
            Value = "51N001",
            Id = Ulid.Empty,
        });
        var result = Handler.IsAllowed(Context, Navdata);
        result.Should().BeFalse();
    }

    [Test]
    [TestCase("51N000X")]
    [TestCase("51S000X")]
    [TestCase("51X000E")]
    [TestCase("51X000W")]
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
    [TestCase("38N054E", 38.0, 54.0)]
    [TestCase("38S054W", -38.0, -54.0)]
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
