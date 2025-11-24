using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class AtcBooking
{
    public required Ulid Id { get; set; }

    public required Ulid UserId { get; set; }
    public User? User { get; set; }

    public required string Callsign { get; set; }

    public required DateTimeOffset BookedAt { get; set; }

    public required DateTimeOffset StartAt { get; set; }

    public required DateTimeOffset EndAt { get; set; }

    public class AtcBookingConfiguration : IEntityTypeConfiguration<AtcBooking>
    {
        public void Configure(EntityTypeBuilder<AtcBooking> builder)
        {
            builder.HasOne(b => b.User)
                .WithMany()
                .HasForeignKey(b => b.UserId);
        }
    }
}
