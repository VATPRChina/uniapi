namespace Net.Vatprc.Uniapi.Models.Atc;

public class Schedule
{
    public long Id { get; set; }

    public string Callsign { get; set; } = null!;

    public decimal UserId { get; set; }

    public DateTime Start { get; set; }

    public DateTime? Finish { get; set; }

    public string? Remark { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual User User { get; set; } = null!;
}
