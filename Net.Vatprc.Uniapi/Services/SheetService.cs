using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Services;

public class SheetService(
    VATPRCContext dbContext
)
{
    public async Task<Sheet?> GetSheetByIdAsync(string sheetId, CancellationToken ct = default)
    {
        return await dbContext.Sheet
          .Where(s => s.Id == sheetId)
          .FirstOrDefaultAsync(ct);
    }
}
