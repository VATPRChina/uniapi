namespace Net.Vatprc.Uniapi.Models.Navdata.Fixes;

public class NdbNavaid : FixWithIdentifier
{
    public NdbNavaid(string icaoCode, string identifier, double latitude, double longitude) : base(icaoCode, identifier, latitude, longitude)
    {
    }
}
