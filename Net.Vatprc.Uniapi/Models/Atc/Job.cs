namespace Net.Vatprc.Uniapi.Models.Atc;

public class Job
{
    public long Id { get; set; }

    public string Queue { get; set; } = null!;

    public string Payload { get; set; } = null!;

    public short Attempts { get; set; }

    public long? ReservedAt { get; set; }

    public long AvailableAt { get; set; }

    public long CreatedAt { get; set; }
}
