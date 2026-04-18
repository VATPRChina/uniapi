namespace Net.Vatprc.Uniapi.Models.Navdata;

public abstract class Leg
{
    public Fix From { get; protected set; }
    public Fix To { get; protected set; }

    public Leg(Fix from, Fix to)
    {
        From = from;
        To = to;
    }
}
