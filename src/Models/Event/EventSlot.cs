using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Event;

public class EventSlot
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public EventAirspace EventAirspace { get; set; } = null!;
    public Ulid EventAirspaceId { get; set; }

    public DateTimeOffset EnterAt { get; set; }

    public DateTimeOffset? LeaveAt { get; set; }

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public EventBooking? Booking { get; set; }

    public string? Callsign { get; set; }
    public string? AircraftTypeIcao { get; set; }

    public class EventSlotConfiguration : IEntityTypeConfiguration<EventSlot>
    {
        public void Configure(EntityTypeBuilder<EventSlot> builder)
        {
            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.UpdatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.HasOne(x => x.EventAirspace)
                .WithMany(x => x.Slots)
                .IsRequired();
        }
    }
}
