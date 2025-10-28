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
        public const string Staff = "staff";
        public const string Volunteer = "volunteer";

        public const string DivisionDirector = "director";

        public const string ControllerTrainingDirector = "controller-training-director";
        public const string ControllerTrainingDirectorAssistant = "controller-training-director-assistant";
        public const string ControllerTrainingInstructor = "controller-training-instructor";
        public const string ControllerTrainingMentor = "controller-training-mentor";
        public const string ControllerTrainingSopEditor = "controller-training-sop-editor";

        public const string OperationDirector = "operation-director";
        public const string OperationDirectorAssistant = "operation-director-assistant";
        public const string OperationSectorEditor = "operation-sector-editor";
        public const string OperationLoaEditor = "operation-loa-editor";

        public const string EventDirector = "event-director";
        public const string EventCoordinator = "event-coordinator";
        public const string EventGraphicsDesigner = "event-graphics-designer";

        public const string TechDirector = "tech-director";
        public const string TechDirectorAssistant = "tech-director-assistant";
        public const string TechAfvFacilityEngineer = "tech-afv-facility-engineer";

        public const string Controller = "controller";
    }

    public static class SpecialRoles
    {
        public const string ApiClient = "api_client";
        public const string User = "user";
    }
}
