using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Services;

public class DiscordRoleMapper(
    DatabaseAdapter databaseAdapter
)
{
    public async Task<IEnumerable<ulong>> GetUserRoles(User user)
    {
        var userRoles = UserRoleService.GetRoleClosure(user.Roles);
        var roles = new HashSet<ulong>();

        foreach (var role in userRoles)
        {
            switch (role)
            {
                case UserRoles.Staff:
                    roles.Add(1246778552506912822);  // VATPRC Division Staff
                    break;
                case UserRoles.Volunteer: break;

                case UserRoles.DivisionDirector:
                    roles.Add(1246778667250483252);  // Division Director
                    break;

                case UserRoles.ControllerTrainingDirector:
                    roles.Add(1246778945446084679); // Controller Training Director
                    roles.Add(1246782637868191805); // Controller Training Department
                    break;
                case UserRoles.ControllerTrainingDirectorAssistant:
                    roles.Add(1246779236224864287); // Assistant Controller Training Director
                    roles.Add(1246782637868191805); // Controller Training Department
                    break;
                case UserRoles.ControllerTrainingInstructor:
                    roles.Add(1246782731606556702); // ATC Instructor
                    roles.Add(1246782637868191805); // Controller Training Department
                    break;
                case UserRoles.ControllerTrainingMentor:
                    roles.Add(1246782801093857281); // ATC Mentor
                    roles.Add(1246782637868191805); // Controller Training Department
                    break;
                case UserRoles.ControllerTrainingSopEditor:
                    roles.Add(1246782637868191805); // Controller Training Department
                    roles.Add(1246782637868191805); // Controller Training Department
                    break;

                case UserRoles.CommunityDirector:
                    roles.Add(1450638583454109808);  // Community & Membership Director
                    roles.Add(1457310145859162163); // Community & Membership Department
                    break;

                case UserRoles.OperationDirector:
                    roles.Add(1246779017936244817);  // Operation Director
                    roles.Add(1246783722229989449);  // Operation Department
                    break;
                case UserRoles.OperationDirectorAssistant:
                    roles.Add(1246779537061187594);  // Assistant Operation Director
                    roles.Add(1246783722229989449);  // Operation Department
                    break;
                case UserRoles.OperationSectorEditor:
                    roles.Add(1246788258394804265);  // Sector Developer
                    roles.Add(1246783722229989449);  // Operation Department
                    break;
                case UserRoles.OperationLoaEditor:
                    roles.Add(1246783722229989449);  // Operation Department
                    roles.Add(1246783722229989449);  // Operation Department
                    break;

                case UserRoles.EventDirector:
                    roles.Add(1246779056104673333);  // Event & Organization Director
                    roles.Add(1246783314833051658);  // Event & Organization Department
                    break;
                case UserRoles.LeadEventCoordinator:
                    roles.Add(1440687176235876493);  // Lead Event Coordinator
                    roles.Add(1246783314833051658);  // Event & Organization Department
                    break;
                case UserRoles.EventCoordinator:
                    roles.Add(1246783370466299944);  // Event Coordinator
                    roles.Add(1246783314833051658);  // Event & Organization Department
                    break;
                case UserRoles.EventGraphicsDesigner:
                    roles.Add(1246783437684084808);  // Graphic Designer
                    roles.Add(1246783314833051658);  // Event & Organization Department
                    break;

                case UserRoles.TechDirector:
                    roles.Add(1246779088832823357);  // Technology Director
                    roles.Add(1246783535407435886);  // Technology Department
                    break;
                case UserRoles.TechDirectorAssistant:
                    roles.Add(1246779633182048379);  // Assistant Technology Director
                    roles.Add(1246783535407435886);  // Technology Department
                    break;
                case UserRoles.TechAfvFacilityEngineer:
                    roles.Add(1246788695131033631);  // AFV Facility Engineer
                    roles.Add(1246783535407435886);  // Technology Department
                    break;

                case UserRoles.Controller: break;

                case UserRoles.ApiClient: break;
                case UserRoles.User: break;
            }
        }

        var status = await databaseAdapter.UserAtcStatus.SingleOrDefaultAsync(s => s.UserId == user.Id);

        var permissions = await databaseAdapter.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .ToListAsync();

        if (status != null)
        {
            if (status.Rating == "OBS")
            {
                roles.Add(1246789733405950012); // OBS - Observer
            }
            else if (status.Rating == "S1")
            {
                roles.Add(1246789698819461164); // S1 - Developing Controller
            }
            else if (status.Rating == "S2")
            {
                roles.Add(1246789698819461164); // S1 - Developing Controller
                roles.Add(1246789524944846958); // S2 - Aerodrome Controller
            }
            else if (status.Rating == "S3")
            {
                roles.Add(1246789698819461164); // S1 - Developing Controller
                roles.Add(1246789524944846958); // S2 - Aerodrome Controller
                roles.Add(1246789486172573717); // S3 - TMA Controller
            }
            else if (status.Rating == "C1")
            {
                roles.Add(1246789698819461164); // S1 - Developing Controller
                roles.Add(1246789524944846958); // S2 - Aerodrome Controller
                roles.Add(1246789486172573717); // S3 - TMA Controller
                roles.Add(1246789431059546132); // C1 - Enroute Controller
            }
            else if (status.Rating == "C3")
            {
                roles.Add(1246789698819461164); // S1 - Developing Controller
                roles.Add(1246789524944846958); // S2 - Aerodrome Controller
                roles.Add(1246789486172573717); // S3 - TMA Controller
                roles.Add(1246789431059546132); // C1 - Enroute Controller
                roles.Add(1246789399606329405); // C3 - Senior Controller
            }

            if (status.IsVisiting)
            {
                roles.Add(1246789977681956884); // Visiting Controller
            }
            else
            {
                roles.Add(1246789867812294737); // Resident Controller
            }

            if (permissions.Any(p => p.State != Models.Atc.UserAtcPermission.UserControllerState.Certified
                && p.State != Models.Atc.UserAtcPermission.UserControllerState.Mentor))
            {
                roles.Add(1246797798796689459); // ATC in Training
            }
        }

        return roles;
    }

    public IEnumerable<ulong> GetAllManagedRoles()
    {
        return [
            1246778552506912822,  // VATPRC Division Staff
            1246778667250483252,  // Division Director
            1246778945446084679, // Controller Training Director
            1246779017936244817,  // Operation Director
            1246779056104673333,  // Event & Organization Director
            1246779088832823357,  // Technology Director
            1246779236224864287, // Assistant Controller Training Director
            1246779537061187594,  // Assistant Operation Director
            1246779633182048379,  // Assistant Technology Director
            1246782637868191805, // Controller Training Department
            1246782731606556702, // ATC Instructor
            1246782801093857281, // ATC Mentor
            1246783314833051658,  // Event & Organization Department
            1246783370466299944,  // Event Coordinator
            1246783437684084808,  // Graphic Designer
            1246783535407435886,  // Technology Department
            1246783722229989449,  // Operation Department
            1246788258394804265,  // Sector Developer
            1246788695131033631,  // AFV Facility Engineer
            1246789399606329405, // C3 - Senior Controller
            1246789431059546132, // C1 - Enroute Controller
            1246789486172573717, // S3 - TMA Controller
            1246789524944846958, // S2 - Aerodrome Controller
            1246789698819461164, // S1 - Developing Controller
            1246789733405950012, // OBS - Observer
            1246789867812294737, // Resident Controller
            1246789977681956884, // Visiting Controller
            1246797798796689459, // ATC in Training
            1440687176235876493,  // Lead Event Coordinator
            1450638583454109808,  // Community & Membership Director
            1457310145859162163, // Community & Membership Department
        ];
    }
}
