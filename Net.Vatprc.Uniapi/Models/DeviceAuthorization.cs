using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class DeviceAuthorization
{
    public Ulid DeviceCode { get; set; }

    public string UserCode { get; set; } = string.Empty;

    public DateTimeOffset ExpiresAt { get; set; }

    [NotMapped]
    public bool IsExpired => ExpiresAt < DateTimeOffset.UtcNow;

    public DateTimeOffset CreatedAt { get; set; }

    public Ulid? UserId { get; set; }
    public User? User { get; set; }

    public class DeviceAuthorizationConfiguration : IEntityTypeConfiguration<DeviceAuthorization>
    {
        public void Configure(EntityTypeBuilder<DeviceAuthorization> builder)
        {
            builder.HasKey(e => e.DeviceCode);

            builder.HasIndex(e => e.UserCode)
                .IsUnique();

            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");
        }
    }
}
