using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Sheet;

public class Sheet
{
    public required string Id { get; set; }

    public required string Name { get; set; }

    public IList<SheetField>? Fields { get; set; }

    public class Configuration : IEntityTypeConfiguration<Sheet>
    {
        public void Configure(EntityTypeBuilder<Sheet> builder)
        {
        }
    }
}
