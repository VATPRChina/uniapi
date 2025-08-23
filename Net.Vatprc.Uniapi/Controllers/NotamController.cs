using Flurl;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// NOTAM information.
/// </summary>
[ApiController, Route("api/notams")]
[AllowAnonymous]
public class NotamController(DiscourseAdapter DiscourseService) : ControllerBase
{
    public record Notam
    {
        public required string Title { get; set; }
        public required string LanguageCode { get; set; }
        public required string Link { get; set; }
    }

    protected IEnumerable<VatprcAtcApiAdapter.Role> FlattenRoles(IEnumerable<VatprcAtcApiAdapter.Role> Roles)
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
