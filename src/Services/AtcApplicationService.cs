using System.Linq.Expressions;
using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Services;

public class AtcApplicationService(Database database)
{
    public async Task<IEnumerable<AtcApplication>> GetApplications(Ulid? userId = null)
    {
        IQueryable<AtcApplication> query = database.AtcApplication.AsQueryable()
            .Include(a => a.User)
            .OrderByDescending(app => app.AppliedAt);

        if (userId != null)
        {
            query = query.Where(app => app.UserId == userId);
        }

        return await query.ToListAsync();
    }

    public async Task<AtcApplication?> GetApplication(Ulid id)
    {
        return await database.AtcApplication
                .Where(a => a.Id == id)
                .Include(a => a.User)
                .Include(a => a.ApplicationFiling)
                    .ThenInclude(af => af!.Answers)
                        .ThenInclude(ans => ans.Field)
                .Include(a => a.ReviewFiling)
                    .ThenInclude(rf => rf!.Answers)
                        .ThenInclude(ans => ans.Field)
                .OrderByDescending(a => a.AppliedAt)
                .SingleOrDefaultAsync();
    }
}
