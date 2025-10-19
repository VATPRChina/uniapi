using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class User
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public string Cid { get; set; } = string.Empty;

    public string FullName { get; set; } = string.Empty;

    public string Email { get; set; } = string.Empty;

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public IList<string> Roles { get; set; } = new List<string>();

    public IEnumerable<RefreshToken> Sessions { get; set; } = null!;

    public class UserConfiguration : IEntityTypeConfiguration<User>
    {
        public void Configure(EntityTypeBuilder<User> builder)
        {
            builder.HasIndex(x => x.Cid)
                .IsUnique();

            builder.Property(x => x.Email)
                .IsRequired(false);

            builder.HasIndex(x => x.Email)
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
    public static class UserRoles
    {
        /// <summary>
        /// Super admin
        /// </summary>
        public const string Admin = "admin";
        /// <summary>
        /// Event coordinator
        /// </summary>
        public const string EventCoordinator = "event_coordinator";
        /// <summary>
        /// Controller
        /// </summary>
        public const string Controller = "controller";

        public const string Editor = "editor";
    }

    public static class SpecialRoles
    {
        public const string ApiClient = "api_client";
        public const string User = "user";
    }
}
