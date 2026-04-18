using Net.Vatprc.Uniapi.Models.Navdata.Legs;

namespace Net.Vatprc.Uniapi.Models.Navdata;

public class Airway
{
    public string Identifier { get; set; }
    public IList<AirwayLeg> Legs { get; set; }

    public Airway(string identifier, IList<AirwayLeg> legs)
    {
        Identifier = identifier;
        Legs = legs;
    }
}
