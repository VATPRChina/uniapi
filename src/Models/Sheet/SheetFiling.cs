using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Sheet;

public class SheetFiling
{
    public Ulid Id { get; set; }

    public string SheetId { get; set; } = default!;
    public Sheet? Sheet { get; set; }

    public Ulid UserId { get; set; }
    public User? User { get; set; }

    public DateTimeOffset FiledAt { get; set; }

    public IEnumerable<SheetFilingAnswer> Answers { get; set; } = null!;

    public class Configuration : IEntityTypeConfiguration<SheetFiling>
    {
        public void Configure(EntityTypeBuilder<SheetFiling> builder)
        {
            builder.HasOne(x => x.Sheet)
                .WithMany()
                .HasForeignKey(x => x.SheetId)
                .IsRequired(true);

            builder.HasOne(x => x.User)
                .WithMany()
                .HasForeignKey(x => x.UserId)
                .IsRequired(true);
        }
    }
}
