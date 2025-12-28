using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Sheet;

public class SheetFiling
{
    public required Ulid Id { get; set; }

    public required string SheetId { get; set; }
    public Sheet? Sheet { get; set; }

    public required Ulid UserId { get; set; }
    public User? User { get; set; }

    public required DateTimeOffset FiledAt { get; set; }

    public IList<SheetFilingAnswer> Answers { get; set; } = [];

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

            builder.Navigation(x => x.Answers).AutoInclude();
        }
    }
}
