using Net.Vatprc.Uniapi.Models.Navdata.Legs;

namespace Net.Vatprc.Uniapi.Models.Navdata;

public class Procedure
{
    public string Identifier { get; set; }
    public IList<ProcedureLeg> Legs { get; set; }

    public Procedure(string identifier, IList<ProcedureLeg> legs)
    {
        Identifier = identifier;
        Legs = legs;
    }
}
