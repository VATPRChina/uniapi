using Microsoft.EntityFrameworkCore.Metadata.Builders;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Models.Event;

public class EventAtcPosition
{
    public Ulid Id { get; set; }

    public Ulid EventId { get; set; }
    public Event? Event { get; set; }

    public required string Callsign { get; set; }

    public required DateTimeOffset StartAt { get; set; }

    public required DateTimeOffset EndAt { get; set; }

    public string? Remarks { get; set; }

    public required string PositionKindId { get; set; }
    public required UserControllerState MinimumControllerState { get; set; }

    public EventAtcPositionBooking? Booking { get; set; }

    public class EventAtcPositionConfiguration : IEntityTypeConfiguration<EventAtcPosition>
    {
        public void Configure(EntityTypeBuilder<EventAtcPosition> builder)
        {
            builder.Property(x => x.Remarks)
                .IsRequired(false);

            builder.HasOne(x => x.Event)
                .WithMany(x => x.AtcPositions)
                .HasForeignKey(x => x.EventId)
                .IsRequired();
        }
    }
}
