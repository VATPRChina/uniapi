using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class UserAtcStatus
{
    public required Ulid UserId { get; set; }

    public User? User { get; set; }

    public required bool IsVisiting { get; set; }

    public required bool IsAbsent { get; set; }

    public class UserAtcStatusConfiguration : IEntityTypeConfiguration<UserAtcStatus>
    {
        public void Configure(EntityTypeBuilder<UserAtcStatus> builder)
        {
            builder.HasKey(e => e.UserId);

            builder.HasOne(e => e.User)
                .WithOne()
                .HasForeignKey<UserAtcStatus>(e => e.UserId);
        }
    }
}
