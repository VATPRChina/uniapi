using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Sheet;

public class SheetFilingAnswer
{
    public string SheetId { get; set; } = default!;
    public uint FieldSequence { get; set; }

    public SheetField? Field { get; set; }

    public Ulid FilingId { get; set; }
    public SheetFiling? Filing { get; set; }

    public string Answer { get; set; } = default!;

    public class Configuration : IEntityTypeConfiguration<SheetFilingAnswer>
    {
        public void Configure(EntityTypeBuilder<SheetFilingAnswer> builder)
        {
            builder.HasKey(x => new { x.SheetId, x.FieldSequence, x.FilingId });

            builder.HasOne(x => x.Field)
                .WithMany()
                .HasForeignKey(x => new { x.SheetId, x.FieldSequence })
                .IsRequired(true);

            builder.HasOne(x => x.Filing)
                .WithMany(x => x.Answers)
                .HasForeignKey(x => x.FilingId)
                .IsRequired(true);
        }
    }
}
