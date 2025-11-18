using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Sheet;

public class SheetField
{
    public required string SheetId { get; set; }
    public Sheet? Sheet { get; set; }

    public required string Id { get; set; }

    public required uint Sequence { get; set; }

    public required string NameZh { get; set; }

    public string? NameEn { get; set; }

    public required SheetFieldKind Kind { get; set; }

    public IEnumerable<string> SingleChoiceOptions { get; set; } = [];

    public bool IsDeleted { get; set; } = false;

    public class Configuration : IEntityTypeConfiguration<SheetField>
    {
        public void Configure(EntityTypeBuilder<SheetField> builder)
        {
            builder.HasKey(x => new { x.SheetId, x.Id });

            builder.HasOne(x => x.Sheet)
                .WithMany(x => x.Fields)
                .HasForeignKey(x => x.SheetId)
                .IsRequired(true);

            builder.Property(x => x.IsDeleted)
                .HasDefaultValue(false);
        }
    }
}
