using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Authorization;
using Flurl;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// NOTAM information.
/// </summary>
[ApiController, Route("api/notams")]
[AllowAnonymous]
public class NotamController(DiscourseService DiscourseService) : ControllerBase
{
    public record Notam
    {
        public required string Title { get; set; }
        public required string LanguageCode { get; set; }
        public required string Link { get; set; }
    }

    protected IEnumerable<VatprcAtcService.Role> FlattenRoles(IEnumerable<VatprcAtcService.Role> Roles)
    {
        return Roles.SelectMany(r => FlattenRoles(r.AllSuperroles)).Concat(Roles);
    }

    [HttpGet]
    [ProducesResponseType<IEnumerable<Notam>>(200)]
    public async Task<IActionResult> GetPermission()
    {
        return Ok((await DiscourseService.GetNotamTopics()).TopicList.Topics.Select(x => new Notam
        {
            Title = x.Title,
            LanguageCode = x.Tags.Contains("english") ? "en" : "zh",
            Link = DiscourseService.Endpoint.AppendPathSegments("/t/topic", x.Id.ToString()),
        }));
    }
}
