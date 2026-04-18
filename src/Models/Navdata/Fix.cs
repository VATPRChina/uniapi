namespace Net.Vatprc.Uniapi.Models.Navdata;

public abstract class Fix
{
    public double Latitude { get; protected set; }
    public double Longitude { get; protected set; }

    public string Type => GetType().Name;
    public virtual string Name => $"{Latitude:F6},{Longitude:F6}";

    public Fix(double latitude, double longitude)
    {
        Latitude = latitude;
        Longitude = longitude;
    }
}

public abstract class FixWithIdentifier : Fix
{
    public string IcaoCode { get; protected set; }
    public string Identifier { get; protected set; }
    public override string Name => Identifier;

    public FixWithIdentifier(string icaoCode, string identifier, double latitude, double longitude) : base(latitude, longitude)
    {
        IcaoCode = icaoCode;
        Identifier = identifier;
    }

    public override bool Equals(object? obj)
    {
        return obj is FixWithIdentifier other
            && IcaoCode == other.IcaoCode
            && Identifier == other.Identifier;
    }

    public override int GetHashCode()
    {
        return HashCode.Combine(Identifier, Latitude, Longitude);
    }
}
