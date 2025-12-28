using Microsoft.EntityFrameworkCore.Metadata.Builders;
using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class Training
{
    public required Ulid Id { get; set; }

    public required string Name { get; set; }

    public required Ulid TrainerId { get; set; }
    public User? Trainer { get; set; }

    public required Ulid TraineeId { get; set; }
    public User? Trainee { get; set; }

    public required DateTimeOffset StartAt { get; set; }

    public required DateTimeOffset EndAt { get; set; }

    public required DateTimeOffset CreatedAt { get; set; }

    public required DateTimeOffset UpdatedAt { get; set; }

    public Ulid? RecordSheetFilingId { get; set; }
    public SheetFiling? RecordSheetFiling { get; set; }

    public class TrainConfiguration : IEntityTypeConfiguration<Training>
    {
        public void Configure(EntityTypeBuilder<Training> builder)
        {
            builder.HasOne(b => b.Trainer)
                .WithMany()
                .HasForeignKey(b => b.TrainerId);

            builder.HasOne(b => b.Trainee)
                .WithMany()
                .HasForeignKey(b => b.TraineeId);

            builder.HasOne(b => b.RecordSheetFiling)
                .WithMany()
                .HasForeignKey(b => b.RecordSheetFilingId);
        }
    }
}
