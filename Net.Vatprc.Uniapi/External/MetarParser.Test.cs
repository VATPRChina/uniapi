namespace Net.Vatprc.Uniapi.External;

[TestFixture]
public class MetarParserTest
{
    [Test]
    public void TestGetMetarTime()
    {
        var time = MetarParser.GetMetarTime("EHAM 011200Z 18010KT 9999 FEW020 SCT030 BKN040 20/10 Q1013");
        time.Day.Should().Be(1);
        time.Hour.Should().Be(12);
        time.Minute.Should().Be(0);

        time = MetarParser.GetMetarTime("EHAM 011230Z 18010KT 9999 FEW020 SCT030 BKN040 20/10 Q1013");
        time.Day.Should().Be(1);
        time.Hour.Should().Be(12);
        time.Minute.Should().Be(30);

        time = MetarParser.GetMetarTime("EHAM 011259Z 18010KT 9999 FEW020 SCT030 BKN040 20/10 Q1013");
        time.Day.Should().Be(1);
        time.Hour.Should().Be(12);
        time.Minute.Should().Be(59);
    }
}
