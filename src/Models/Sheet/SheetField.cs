using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Sheet;

public class SheetField
{
    public string SheetId { get; set; } = default!;
    public Sheet Sheet { get; set; } = default!;

    public uint Sequence { get; set; }

    public string NameZh { get; set; } = default!;

    public string NameEn { get; set; } = default!;

    public SheetFieldKind Kind { get; set; }

    public IEnumerable<string> SingleChoiceOptions { get; set; } = [];

    public class Configuration : IEntityTypeConfiguration<SheetField>
    {
        public void Configure(EntityTypeBuilder<SheetField> builder)
        {
            builder.HasKey(x => new { x.SheetId, x.Sequence });

            builder.HasOne(x => x.Sheet)
                .WithMany(x => x.Fields)
                .HasForeignKey(x => x.SheetId)
                .IsRequired(true);
        }
    }
}
