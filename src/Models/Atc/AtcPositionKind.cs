namespace Net.Vatprc.Uniapi.Models.Atc;

public class AtcPositionKind
{
    public required string Id { get; set; }

    public required string NameZh { get; set; }

    public required string NameEn { get; set; }

    public bool IsTrainable { get; set; } = true;
}
