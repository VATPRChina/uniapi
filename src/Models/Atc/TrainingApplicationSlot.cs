using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class TrainingApplicationSlot
{
    public required Ulid Id { get; set; }

    public required Ulid ApplicationId { get; set; }
    public TrainingApplication? Application { get; set; }

    public required DateTimeOffset StartAt { get; set; }

    public required DateTimeOffset EndAt { get; set; }

    public class TrainingApplicationSlotConfiguration : IEntityTypeConfiguration<TrainingApplicationSlot>
    {
        public void Configure(EntityTypeBuilder<TrainingApplicationSlot> builder)
        {
            builder.HasKey(b => b.Id);

            builder.HasOne(b => b.Application)
                .WithMany(a => a.Slots)
                .HasForeignKey(b => b.ApplicationId);
        }
    }
}
