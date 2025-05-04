namespace Net.Vatprc.Uniapi.Utils;

[TestFixture]
public class DateTimeUtilTest
{
    [Test]
    public void TestRoundUp()
    {
        DateTimeOffset.Parse("2023-10-01T12:14:56Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T12:15:00Z"));
        DateTimeOffset.Parse("2023-10-01T12:15:00Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T12:15:00Z"));

        DateTimeOffset.Parse("2023-10-01T12:24:56Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T12:30:00Z"));
        DateTimeOffset.Parse("2023-10-01T12:30:00Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T12:30:00Z"));

        DateTimeOffset.Parse("2023-10-01T12:34:56Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T12:45:00Z"));
        DateTimeOffset.Parse("2023-10-01T12:45:00Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T12:45:00Z"));

        DateTimeOffset.Parse("2023-10-01T12:54:56Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T13:00:00Z"));
        DateTimeOffset.Parse("2023-10-01T13:00:00Z").RoundUp(TimeSpan.FromMinutes(15))
            .Should().Be(DateTimeOffset.Parse("2023-10-01T13:00:00Z"));
    }
}
