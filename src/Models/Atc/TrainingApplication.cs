using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class TrainingApplication
{
    public required Ulid Id { get; set; }

    public required Ulid TraineeId { get; set; }
    public User? Trainee { get; set; }

    public required string Name { get; set; }

    public Ulid? TrainId { get; set; }
    public Training? Train { get; set; }

    public required DateTimeOffset CreatedAt { get; set; }

    public required DateTimeOffset UpdatedAt { get; set; }

    public DateTimeOffset? DeletedAt { get; set; }

    public IEnumerable<TrainingApplicationSlot>? Slots { get; set; }

    public class TrainApplicationConfiguration : IEntityTypeConfiguration<TrainingApplication>
    {
        public void Configure(EntityTypeBuilder<TrainingApplication> builder)
        {
            builder.HasKey(b => b.Id);

            builder.HasOne(b => b.Trainee)
                .WithMany()
                .HasForeignKey(b => b.TraineeId);

            builder.HasOne(b => b.Train)
                .WithOne()
                .HasForeignKey<TrainingApplication>(b => b.TrainId);

            builder.Property(b => b.DeletedAt)
                .IsRequired(false);
        }
    }
}
