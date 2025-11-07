using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class UserAtcPermission
{
    public Ulid UserId { get; set; }

    public User? User { get; set; }

    public string PositionKindId { get; set; } = null!;

    public UserControllerState State { get; set; }

    public DateTimeOffset? SoloExpiresAt { get; set; }

    public bool CanAccessMoodle => true;

    public bool CanOnline => State is UserControllerState.UnderMentor
        or UserControllerState.Solo
        or UserControllerState.Certified;

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
