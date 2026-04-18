namespace Net.Vatprc.Uniapi.Models.Navdata.Fixes;

public class VhfNavaid : FixWithIdentifier
{
    public VhfNavaid(string icaoCode, string identifier, double latitude, double longitude) : base(icaoCode, identifier, latitude, longitude)
    {
    }
}
