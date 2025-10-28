using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Services;

[TestFixture]
public class UserRoleServiceTest
{
    [Test]
    public void GetRoleClosure_ShouldIncludeTransitive()
    {
        UserRoleService.GetRoleClosure([User.UserRoles.ControllerTrainingInstructor])
            .Should().BeEquivalentTo([
                User.UserRoles.ControllerTrainingInstructor,
                User.UserRoles.Volunteer,
                User.UserRoles.ControllerTrainingMentor,
            ]);
    }

    [Test]
    public void GetRoleClosure_ShouldForEachInput()
    {
        UserRoleService.GetRoleClosure([
            User.UserRoles.ControllerTrainingInstructor,
            User.UserRoles.TechDirector,
        ]).Should().BeEquivalentTo([
                User.UserRoles.Staff,
                User.UserRoles.Volunteer,
                User.UserRoles.ControllerTrainingInstructor,
                User.UserRoles.ControllerTrainingMentor,
                User.UserRoles.TechDirector,
                User.UserRoles.TechDirectorAssistant,
                User.UserRoles.TechAfvFacilityEngineer,
            ]);
    }
}
