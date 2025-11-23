using Net.Vatprc.Uniapi.Models.Atc;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Services;

public class AtcPositionStatusService
{
    public bool IsStatusSatifyMinimum(UserAtcPermission permission, UserControllerState minimumState)
    {
        return permission.State switch
        {
            UserControllerState.Student => minimumState is UserControllerState.Student,
            UserControllerState.UnderMentor => minimumState is UserControllerState.Student or UserControllerState.UnderMentor,
            UserControllerState.Solo => minimumState is UserControllerState.Student
                or UserControllerState.UnderMentor
                or UserControllerState.Solo
                && (permission.SoloExpiresAt == null || permission.SoloExpiresAt > DateTimeOffset.UtcNow),
            UserControllerState.Certified => minimumState is UserControllerState.Student
                or UserControllerState.UnderMentor
                or UserControllerState.Solo
                or UserControllerState.Certified,
            _ => throw new NotImplementedException("Invalid permission state: " + permission.State),
        };
    }
}
