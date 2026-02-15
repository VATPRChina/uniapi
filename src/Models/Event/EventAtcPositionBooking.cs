using Microsoft.EntityFrameworkCore.Metadata.Builders;
using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Models.Event;

public class EventAtcPositionBooking
{
    public required Ulid EventAtcPositionId { get; set; }
    public EventAtcPosition? EventAtcPosition { get; set; }

    public required Ulid UserId { get; set; }
    public User? User { get; set; }

    public required Ulid? AtcBookingId { get; set; }
    public AtcBooking? AtcBooking { get; set; }

    public required DateTimeOffset CreatedAt { get; set; }

    public class EventAtcPositionBookingConfiguration : IEntityTypeConfiguration<EventAtcPositionBooking>
    {
        public void Configure(EntityTypeBuilder<EventAtcPositionBooking> builder)
        {
            builder.HasKey(x => x.EventAtcPositionId);

            builder.HasOne(x => x.EventAtcPosition)
                .WithOne(x => x.Booking)
                .HasForeignKey<EventAtcPositionBooking>(x => x.EventAtcPositionId)
                .IsRequired();

            builder.HasOne(x => x.User)
                .WithMany()
                .HasForeignKey(x => x.UserId)
                .IsRequired();

            builder.HasOne(x => x.AtcBooking)
                .WithOne()
                .HasForeignKey<EventAtcPositionBooking>(x => x.AtcBookingId);
        }
    }
}
