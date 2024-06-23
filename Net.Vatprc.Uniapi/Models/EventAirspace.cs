using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class EventAirspace
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public Event Event { get; set; } = null!;
    public Ulid EventId { get; set; }

    public string Name { get; set; } = string.Empty;

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public IEnumerable<EventSlot> Slots { get; set; } = null!;

    public class EventAirspaceConfiguration : IEntityTypeConfiguration<EventAirspace>
    {
        public void Configure(EntityTypeBuilder<EventAirspace> builder)
        {
            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.UpdatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP")
                .ValueGeneratedOnAddOrUpdate();

            builder.HasOne(x => x.Event)
                .WithMany(x => x.Airspaces)
                .IsRequired();
        }
    }
}
