namespace Net.Vatprc.Uniapi.Models.Navdata.Fixes;

public class Waypoint : FixWithIdentifier
{
    public Waypoint(string icaoCode, string identifier, double latitude, double longitude) : base(icaoCode, identifier, latitude, longitude)
    {
    }
}
