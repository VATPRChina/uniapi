using Moq;
using Net.Vatprc.Uniapi.Services.FlightPlan;
using Net.Vatprc.Uniapi.Services.FlightPlan.Lexing;
using Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

namespace Net.Vatprc.Uniapi.Test;

[TestFixture]
public class InitialSpeedAndAltitudeTokenHandlerTests
{
    protected Mock<INavdataProvider> mockNavdataProvider = new Mock<INavdataProvider>();

    [TestCase("M080F045")]
    [TestCase("K0939S0980")]
    [TestCase("VFRA300")]
    public async Task Resolve_ValidSpeedAndAltitude_ReturnsTrue(string input)
    {
        var handler = new InitialSpeedAndAltitudeTokenHandler();
        var mockContext = new Mock<ILexerContext>();
        mockContext.Setup(c => c.CurrentSegmentIndex).Returns(0);
        mockContext.Setup(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = input,
            Id = string.Empty,
            Geo = null,
        });

        var result = await handler.Resolve(mockContext.Object, mockNavdataProvider.Object);

        result.Should().BeTrue();
    }

    [TestCase("X123F045")]
    [TestCase("M080X123")]
    public async Task Resolve_InvalidInput_ReturnsFalse(string input)
    {
        var handler = new InitialSpeedAndAltitudeTokenHandler();
        var mockContext = new Mock<ILexerContext>();
        mockContext.Setup(c => c.CurrentSegmentIndex).Returns(0);
        mockContext.Setup(c => c.CurrentSegment).Returns(new RouteToken
        {
            Kind = RouteTokenKind.UNKNOWN,
            Value = input,
            Id = string.Empty,
            Geo = null,
        });

        var result = await handler.Resolve(mockContext.Object, mockNavdataProvider.Object);

        result.Should().BeFalse();
    }
}
