using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class RefreshToken
{
    public Ulid Token { get; set; } = Ulid.NewUlid();

    public User User { get; set; } = null!;
    public Ulid UserId { get; set; }

    public DateTimeOffset UserUpdatedAt { get; set; }

    public DateTimeOffset ExpiresIn { get; set; }

    public DateTimeOffset CreatedAt { get; set; }

    public Ulid? AuthzCode { get; set; }

    public string ClientId { get; set; } = string.Empty;

    // TODO: public Ulid? GroupId { get; set; }
    // Group id is used for detecting refresh token mis-reuse.

    public class SessionConfiguration : IEntityTypeConfiguration<RefreshToken>
    {
        public void Configure(EntityTypeBuilder<RefreshToken> builder)
        {
            builder.ToTable("session");

            builder.HasKey(x => x.Token);

            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.HasOne(x => x.User)
                .WithMany(x => x.Sessions);

            builder.Property(x => x.AuthzCode)
                .HasColumnName("code");
        }
    }
}
