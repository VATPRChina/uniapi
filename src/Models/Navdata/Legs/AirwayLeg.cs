namespace Net.Vatprc.Uniapi.Models.Navdata.Legs;

public class AirwayLeg : Leg
{
    public string Identifier { get; set; }
    public new FixWithIdentifier From => (FixWithIdentifier)base.From;
    public new FixWithIdentifier To => (FixWithIdentifier)base.To;
    public AirwayDirection Direction { get; set; }

    public AirwayLeg(
        FixWithIdentifier from,
        FixWithIdentifier to,
        string identifier,
        AirwayDirection direction) : base(from, to)
    {
        Identifier = identifier;
        Direction = direction;
    }

    public enum AirwayDirection
    {
        FORWARD,
        BACKWARD,
        BOTH,
    }
}
