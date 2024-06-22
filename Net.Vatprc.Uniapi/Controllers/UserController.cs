using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using PdfSharp.Pdf;
using PdfSharp.Pdf.IO;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/users")]
public class UserController : ControllerBase
{
    private VATPRCContext DbContext { get; init; }

    public UserController(VATPRCContext dbContext)
    {
        DbContext = dbContext;
    }

    public record UserDto
    {
        public Ulid Id { get; init; }
        public string Cid { get; init; }
        public string FullName { get; init; }
        public DateTimeOffset CreatedAt { get; init; }
        public DateTimeOffset UpdatedAt { get; init; }

        public UserDto(User user)
        {
            Id = user.Id;
            Cid = user.Cid;
            FullName = user.FullName;
            CreatedAt = user.CreatedAt;
            UpdatedAt = user.UpdatedAt;
        }
    }

    [HttpGet]
    [Authorize(Roles = Models.User.UserRoles.Admin)]
    public async Task<IEnumerable<UserDto>> List()
    {
        return await DbContext.User.Select(x => new UserDto(x)).ToListAsync();
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = Models.User.UserRoles.Admin)]
    public async Task<UserDto> Get(Ulid id)
    {
        return new UserDto(await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id));
    }
}
