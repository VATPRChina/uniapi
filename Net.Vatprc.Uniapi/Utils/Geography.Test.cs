using NUnit.Framework;

namespace Net.Vatprc.Uniapi.Utils;

[TestFixture]
public class GeographyTest
{
    protected const double PRECISION = 0.5 / 3600;

    [Test]
    public void TestParseCaacCoordinate()
    {
        Geography.ParseCaacCoordinate("N400421").Should().BeApproximately(40.0725, PRECISION);
        Geography.ParseCaacCoordinate("E1163551").Should().BeApproximately(116.5975, PRECISION);
        Geography.ParseCaacCoordinate("N393003").Should().BeApproximately(39.500833333333, PRECISION);
        Geography.ParseCaacCoordinate("E1162356").Should().BeApproximately(116.39888888889, PRECISION);
        Geography.ParseCaacCoordinate("N384454").Should().BeApproximately(38.748333333333, PRECISION);
        Geography.ParseCaacCoordinate("E1053503").Should().BeApproximately(105.58416666667, PRECISION);
        Geography.ParseCaacCoordinate("N391331").Should().BeApproximately(39.225277777778, PRECISION);
        Geography.ParseCaacCoordinate("E1013258").Should().BeApproximately(101.54944444444, PRECISION);
    }

    [Test]
    public void TestDistanceBetweenPoints()
    {
        Geography.DistanceBetweenPoints(
            40.0725, 116.5975,
            39.500833333333, 116.39888888889
        ).Should().BeApproximately(35.53, 0.05);
    }
}
