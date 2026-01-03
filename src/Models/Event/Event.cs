using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Event;

public class Event
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public string Title { get; set; } = string.Empty;

    public string Description { get; set; } = string.Empty;

    public DateTimeOffset StartAt { get; set; }

    public DateTimeOffset EndAt { get; set; }

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public DateTimeOffset StartBookingAt { get; set; }

    public DateTimeOffset EndBookingAt { get; set; }

    public DateTimeOffset? StartAtcBookingAt { get; set; }

    public string? ImageUrl { get; set; }

    public IEnumerable<EventAirspace>? Airspaces { get; set; }

    public IEnumerable<EventAtcPosition>? AtcPositions { get; set; }

    public bool IsInBookingPeriod
    {
        get => DateTimeOffset.Now > StartBookingAt && DateTimeOffset.Now < EndBookingAt;
    }

    public bool IsInAtcBookingPeriod
    {
        get => StartAtcBookingAt == null || (DateTimeOffset.Now > StartAtcBookingAt);
    }

    public class EventConfiguration : IEntityTypeConfiguration<Event>
    {
        public void Configure(EntityTypeBuilder<Event> builder)
        {
            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.UpdatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.StartAtcBookingAt)
                .IsRequired(false);

            builder.Property(x => x.ImageUrl)
                .IsRequired(false);
        }
    }
}
