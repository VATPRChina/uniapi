namespace Net.Vatprc.Uniapi.Models.Atc;

public class EventsPositionsBooking
{
    public decimal EventsPositionId { get; set; }

    public decimal Controller { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual User ControllerNavigation { get; set; } = null!;
}
