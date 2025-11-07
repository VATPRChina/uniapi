namespace Net.Vatprc.Uniapi.Models;

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

    public const string ApiClient = "api-client";
    public const string User = "user";

    public static IEnumerable<string> AllRoles => typeof(UserRoles).GetFields()
            .Where(f => f.IsStatic && f.IsLiteral && f.FieldType == typeof(string))
            .Select(f => f.GetValue(null)?.ToString() ?? string.Empty)
            .ToList();
}
