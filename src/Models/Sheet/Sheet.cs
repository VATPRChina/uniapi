using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Sheet;

public class Sheet
{
    public string Id { get; set; } = default!;

    public string Name { get; set; } = default!;

    public IEnumerable<SheetField> Fields { get; set; } = null!;

    public class Configuration : IEntityTypeConfiguration<Sheet>
    {
        public void Configure(EntityTypeBuilder<Sheet> builder)
        {
        }
    }
}
