namespace Net.Vatprc.Uniapi.Models.Navdata.Fixes;

public class UnknownFix : Fix
{
    public UnknownFix(double latitude, double longitude) : base(latitude, longitude)
    {
    }

    public override bool Equals(object? obj)
    {
        return false;
    }

    public override int GetHashCode()
    {
        return base.GetHashCode();
    }
}
