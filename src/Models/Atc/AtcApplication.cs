using Microsoft.EntityFrameworkCore.Metadata.Builders;
using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class AtcApplication
{
    public Ulid Id { get; set; }

    public Ulid UserId { get; set; }
    public User? User { get; set; }

    public Ulid ApplicationFilingId { get; set; }
    public SheetFiling? ApplicationFiling { get; set; }

    public Ulid? ReviewFilingId { get; set; }
    public SheetFiling? ReviewFiling { get; set; }

    public DateTimeOffset AppliedAt { get; set; }

    public AtcApplicationStatus Status { get; set; }

    public class AtcApplicationConfiguration : IEntityTypeConfiguration<AtcApplication>
    {
        public void Configure(EntityTypeBuilder<AtcApplication> builder)
        {
            builder.HasOne(e => e.User)
                .WithMany()
                .HasForeignKey(e => e.UserId);

            builder.HasOne(e => e.ApplicationFiling)
                .WithMany()
                .HasForeignKey(e => e.ApplicationFilingId);

            builder.HasOne(e => e.ReviewFiling)
                .WithMany()
                .HasForeignKey(e => e.ReviewFilingId);

            builder.Property(e => e.Status)
                .HasConversion<string>();
        }
    }
}
