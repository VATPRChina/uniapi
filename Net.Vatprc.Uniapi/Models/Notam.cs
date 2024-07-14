using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class Notam
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public string Title { get; set; } = string.Empty;

    public string Description { get; set; } = string.Empty;

    public DateTimeOffset EffectiveFrom { get; set; }

    public DateTimeOffset ExpireAfter { get; set; }

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public class NotamConfiguration : IEntityTypeConfiguration<Notam>
    {
        public void Configure(EntityTypeBuilder<Notam> builder)
        {
            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.UpdatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP")
                .ValueGeneratedOnAddOrUpdate();
        }
    }
}
