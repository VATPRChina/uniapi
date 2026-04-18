namespace Net.Vatprc.Uniapi.Models.Navdata.Fixes;

public class GeoPoint : Fix
{
    public GeoPoint(double latitude, double longitude) : base(latitude, longitude)
    {
    }

    public override bool Equals(object? obj)
    {
        return obj is GeoPoint point &&
            Latitude == point.Latitude &&
            Longitude == point.Longitude;
    }

    public override int GetHashCode()
    {
        return base.GetHashCode();
    }
}
