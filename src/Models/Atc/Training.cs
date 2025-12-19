using Microsoft.EntityFrameworkCore.Metadata.Builders;
using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class Training
{
    public required Ulid Id;

    public required string Name;

    public required Ulid TrainerId;
    public User? Trainer { get; set; }

    public required Ulid TraineeId;
    public User? Trainee { get; set; }

    public required DateTimeOffset StartAt;

    public required DateTimeOffset EndAt;

    public required DateTimeOffset CreatedAt;

    public required DateTimeOffset UpdatedAt;

    public Ulid? RecordSheetFilingId;
    public SheetFiling? RecordSheetFiling { get; set; }

    public class TrainConfiguration : IEntityTypeConfiguration<Training>
    {
        public void Configure(EntityTypeBuilder<Training> builder)
        {
            builder.HasKey(b => b.Id);

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
