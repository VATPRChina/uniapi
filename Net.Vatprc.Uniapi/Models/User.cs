using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class User
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public string Cid { get; set; } = string.Empty;

    public string FullName { get; set; } = string.Empty;

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public IEnumerable<UserRole> Roles { get; set; } = [];

    public IEnumerable<Session> Sessions { get; set; } = null!;

    public class UserConfiguration : IEntityTypeConfiguration<User>
    {
        public void Configure(EntityTypeBuilder<User> builder)
        {
            builder.HasIndex(x => x.Cid)
                .IsUnique();

            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.UpdatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP")
                .ValueGeneratedOnAddOrUpdate();
        }
    }

    /// <summary>
    /// Roles, which controls permission
    /// </summary>
    public enum UserRole
    {
        /// <summary>
        /// Super admin
        /// </summary>
        Admin,
        /// <summary>
        /// Event coordination
        /// </summary>
        EventCoordinator,
        /// <summary>
        /// ATC
        /// </summary>
        ATC,
    }
}
