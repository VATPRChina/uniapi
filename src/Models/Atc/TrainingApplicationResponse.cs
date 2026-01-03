using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class TrainingApplicationResponse
{
    public required Ulid Id { get; set; }

    public required Ulid ApplicationId { get; set; }
    public TrainingApplication? Application { get; set; }

    public required Ulid TrainerId { get; set; }
    public User? Trainer { get; set; }

    public required Ulid? SlotId { get; set; }
    public TrainingApplicationSlot? Slot { get; set; }

    public required string Comment { get; set; }

    public required DateTimeOffset CreatedAt { get; set; }

    public required DateTimeOffset UpdatedAt { get; set; }

    public class TrainApplicationResponseConfiguration : IEntityTypeConfiguration<TrainingApplicationResponse>
    {
        public void Configure(EntityTypeBuilder<TrainingApplicationResponse> builder)
        {
            builder.HasKey(b => b.Id);

            builder.HasOne(b => b.Application)
                .WithMany()
                .HasForeignKey(b => b.ApplicationId);

            builder.HasOne(b => b.Trainer)
                .WithMany()
                .HasForeignKey(b => b.TrainerId);

            builder.HasOne(b => b.Slot)
                .WithOne()
                .HasForeignKey<TrainingApplicationResponse>(b => b.SlotId);
        }
    }
}
