using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController]
[Route("api/sheets")]
[Authorize(Roles = UserRoles.Staff)]
public class SheetsController(
    SheetService sheetService
) : ControllerBase
{
    [HttpGet]
    public async Task<IEnumerable<SheetDto>> List(CancellationToken ct)
    {
        var sheets = await sheetService.GetSheetsAsync(ct);
        return sheets.Select(SheetDto.From);
    }

    [HttpGet("{sheetId}")]
    [ApiError.Has<ApiError.SheetNotFound>]
    public async Task<SheetDto> Get(string sheetId, CancellationToken ct)
    {
        var sheet = await sheetService.GetSheetByIdAsync(sheetId, ct)
            ?? throw new ApiError.SheetNotFound(sheetId);
        return SheetDto.From(sheet);
    }

    [HttpPut("{sheetId}")]
    [ApiError.Has<ApiError.SheetFieldDuplicateId>]
    [ApiError.Has<ApiError.SheetSingleChoiceOptionsMissing>]
    public async Task<SheetDto> Upsert(string sheetId, [FromBody] SheetSaveRequest request, CancellationToken ct)
    {
        var duplicateFieldId = request.Fields
            .GroupBy(field => field.Id)
            .Where(group => group.Count() > 1)
            .Select(group => group.Key)
            .FirstOrDefault();
        if (duplicateFieldId != null)
        {
            throw new ApiError.SheetFieldDuplicateId(sheetId, duplicateFieldId);
        }

        try
        {
            var sheet = await sheetService.SetSheetAsync(
                sheetId,
                request.Name,
                request.Fields.Select(field => new SheetField
                {
                    SheetId = sheetId,
                    Id = field.Id,
                    Sequence = field.Sequence,
                    NameZh = field.NameZh,
                    NameEn = field.NameEn,
                    Kind = field.Kind,
                    SingleChoiceOptions = field.SingleChoiceOptions ?? [],
                    DescriptionZh = field.DescriptionZh,
                    DescriptionEn = field.DescriptionEn,
                }),
                ct);

            return SheetDto.From(sheet);
        }
        catch (SheetService.SingleChoiceOptionMissingException ex)
        {
            throw new ApiError.SheetSingleChoiceOptionsMissing(sheetId, ex.FieldId);
        }
    }
}
