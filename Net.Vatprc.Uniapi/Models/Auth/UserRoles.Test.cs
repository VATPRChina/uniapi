namespace Net.Vatprc.Uniapi.Models.Auth;

public class UserTest
{
    [Test]
    public void UserRoles_AllRoles_ShouldContainAll()
    {
        var definedRoles = new HashSet<string>
        {
            UserRoles.Staff,
            UserRoles.Volunteer,
            UserRoles.DivisionDirector,
            UserRoles.ControllerTrainingDirector,
            UserRoles.ControllerTrainingDirectorAssistant,
            UserRoles.ControllerTrainingInstructor,
            UserRoles.ControllerTrainingMentor,
            UserRoles.ControllerTrainingSopEditor,
            UserRoles.OperationDirector,
            UserRoles.OperationDirectorAssistant,
            UserRoles.OperationSectorEditor,
            UserRoles.OperationLoaEditor,
            UserRoles.EventDirector,
            UserRoles.EventCoordinator,
            UserRoles.EventGraphicsDesigner,
            UserRoles.TechDirector,
            UserRoles.TechDirectorAssistant,
            UserRoles.TechAfvFacilityEngineer,
            UserRoles.Controller,
        };
        UserRoles.AllRoles.Should().BeEquivalentTo(definedRoles);
    }
}
