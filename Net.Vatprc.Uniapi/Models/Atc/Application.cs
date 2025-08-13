namespace Net.Vatprc.Uniapi.Models.Atc;

public class Application
{
    public long Id { get; set; }

    public decimal Applicant { get; set; }

    public bool Accepted { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public DateTime? LastReviewedAt { get; set; }

    public DateTime? ProcessedAt { get; set; }
}
