using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Services;

[TestFixture]
public class UserRoleServiceTest
{
    [Test]
    public void GetRoleClosure_ShouldIncludeTransitive()
    {
        UserRoleService.GetRoleClosure([UserRoles.ControllerTrainingInstructor])
            .Should().BeEquivalentTo([
                UserRoles.ControllerTrainingInstructor,
                UserRoles.Volunteer,
                UserRoles.ControllerTrainingMentor,
            ]);
    }

    [Test]
    public void GetRoleClosure_ShouldForEachInput()
    {
        UserRoleService.GetRoleClosure([
            UserRoles.ControllerTrainingInstructor,
            UserRoles.TechDirector,
        ]).Should().BeEquivalentTo([
                UserRoles.Staff,
                UserRoles.Volunteer,
                UserRoles.ControllerTrainingInstructor,
                UserRoles.ControllerTrainingMentor,
                UserRoles.TechDirector,
                UserRoles.TechDirectorAssistant,
                UserRoles.TechAfvFacilityEngineer,
            ]);
    }
}
