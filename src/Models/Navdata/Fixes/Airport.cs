namespace Net.Vatprc.Uniapi.Models.Navdata.Fixes;

public class Airport : FixWithIdentifier
{
    public Airport(string icaoCode, string identifier, double latitude, double longitude) : base(icaoCode, identifier, latitude, longitude)
    {
    }
}
