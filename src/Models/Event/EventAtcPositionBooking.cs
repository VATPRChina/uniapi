using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Event;

public class EventAtcPositionBooking
{
    public Ulid EventAtcPositionId { get; set; }
    public EventAtcPosition? EventAtcPosition { get; set; }

    public Ulid UserId { get; set; }
    public User? User { get; set; }

    public DateTimeOffset CreatedAt { get; set; }

    public class EventAtcPositionBookingConfiguration : IEntityTypeConfiguration<EventAtcPositionBooking>
    {
        public void Configure(EntityTypeBuilder<EventAtcPositionBooking> builder)
        {
            builder.HasKey(x => x.EventAtcPositionId);

            builder.HasOne(x => x.EventAtcPosition)
                .WithOne(x => x.Booking)
                .HasForeignKey<EventAtcPositionBooking>(x => x.EventAtcPositionId)
                .IsRequired();
        }
    }
}
