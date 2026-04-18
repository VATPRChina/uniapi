namespace Net.Vatprc.Uniapi.Models.Navdata.Legs;

public class ProcedureLeg : Leg
{
    public string Identifier { get; set; }

    public ProcedureLeg(Fix from, Fix to, string identifier) : base(from, to)
    {
        Identifier = identifier;
    }
}
