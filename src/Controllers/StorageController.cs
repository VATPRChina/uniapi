using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController, Route("api/storage")]
public class StorageController(SmmsAdapter smmsAdapter) : ControllerBase
{
    [HttpPost("images")]
    [Authorize(Roles = Models.UserRoles.Volunteer)]
    public async Task<UploadImageResponse> UploadImage(IFormFile image, CancellationToken ct = default)
    {
        if (image == null || image.Length == 0)
        {
            throw new ApiError.BadRequest("No image file provided.");
        }

        await using var stream = image.OpenReadStream();
        var imageUrl = await smmsAdapter.UploadImageAsync(stream, image.FileName, ct);
        return new UploadImageResponse(imageUrl);
    }

    public record class UploadImageResponse(string Url);
}
