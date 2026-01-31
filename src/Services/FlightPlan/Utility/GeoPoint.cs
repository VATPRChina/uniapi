using Arinc424;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

public class GeoPoint : Geo
{
    public GeoPoint(double latitude, double longitude, string? source = null)
    {
        Coordinates = new Coordinates(latitude, longitude);
        Code = string.Empty;
        Date = 0;
        Number = -1;
        Source = source ?? $"{latitude},{longitude}";
    }
}
