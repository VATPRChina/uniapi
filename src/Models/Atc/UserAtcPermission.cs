using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class UserAtcPermission
{
    public required Ulid UserId { get; set; }

    public User? User { get; set; }

    public required string PositionKindId { get; set; }

    public UserControllerState State { get; set; } = UserControllerState.Student;

    public DateTimeOffset? SoloExpiresAt { get; set; }

    public bool CanAccessMoodle => true;

    public bool CanOnline => State is UserControllerState.UnderMentor
        or UserControllerState.Solo
        or UserControllerState.Certified
        && (SoloExpiresAt == null || SoloExpiresAt > DateTimeOffset.UtcNow);

    public bool CanRequestMentorSession => State is UserControllerState.UnderMentor
        or UserControllerState.Solo;

    public enum UserControllerState
    {
        Student,
        UnderMentor,
        Solo,
        Certified,
    }

    public class UserAtcPermissionConfiguration : IEntityTypeConfiguration<UserAtcPermission>
    {
        public void Configure(EntityTypeBuilder<UserAtcPermission> builder)
        {
            builder.HasKey(e => new { e.UserId, e.PositionKindId });

            builder.Property(x => x.State)
                .HasConversion<string>();
        }
    }
}
