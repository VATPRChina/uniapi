using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class EventBooking
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public User User { get; set; } = null!;
    public Ulid UserId { get; set; }

    public EventSlot EventSlot { get; set; } = null!;
    public Ulid EventSlotId { get; set; }

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public class EventBookingConfiguration : IEntityTypeConfiguration<EventBooking>
    {
        public void Configure(EntityTypeBuilder<EventBooking> builder)
        {
            builder.HasIndex(x => x.EventSlotId)
                .IsUnique();

            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.UpdatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP")
                .ValueGeneratedOnAddOrUpdate();

            builder.HasOne(x => x.EventSlot)
                .WithOne(x => x.Booking)
                .IsRequired();
        }
    }
}
