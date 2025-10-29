namespace Net.Vatprc.Uniapi.Models.Auth;

public class UserTest
{
    [Test]
    public void UserRoles_AllRoles_ShouldContainAll()
    {
        var definedRoles = new HashSet<string>
        {
            User.UserRoles.Staff,
            User.UserRoles.Volunteer,
            User.UserRoles.DivisionDirector,
            User.UserRoles.ControllerTrainingDirector,
            User.UserRoles.ControllerTrainingDirectorAssistant,
            User.UserRoles.ControllerTrainingInstructor,
            User.UserRoles.ControllerTrainingMentor,
            User.UserRoles.ControllerTrainingSopEditor,
            User.UserRoles.OperationDirector,
            User.UserRoles.OperationDirectorAssistant,
            User.UserRoles.OperationSectorEditor,
            User.UserRoles.OperationLoaEditor,
            User.UserRoles.EventDirector,
            User.UserRoles.EventCoordinator,
            User.UserRoles.EventGraphicsDesigner,
            User.UserRoles.TechDirector,
            User.UserRoles.TechDirectorAssistant,
            User.UserRoles.TechAfvFacilityEngineer,
            User.UserRoles.Controller,
        };
        User.UserRoles.AllRoles.Should().BeEquivalentTo(definedRoles);
    }
}
