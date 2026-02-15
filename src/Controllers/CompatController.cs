using System.Net;
using System.Text.RegularExpressions;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Dto;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Compatibility with existing services.
/// </summary>
[ApiController, Route("api/compat")]
[AllowAnonymous]
public partial class CompatController(
    VatsimAdapter VatsimService,
    MetarAdapter MetarService,
    TrackAudioAdapter TrackAudioService,
    DatabaseAdapter database,
    VplaafAdapter vplaafAdapter) : ControllerBase
{

    [GeneratedRegex("^(Z[BSGUHWJPLYM][A-Z0-9]{2}(_[A-Z0-9]*)?_(DEL|GND|TWR|APP|DEP|CTR))|(PRC_FSS)$")]
    protected static partial Regex vatprcControllerRegexp();
    [GeneratedRegex("^Z[BMSPGJYWLH][A-Z]{2}")]
    protected static partial Regex vatprcAirportRegexp();

    [HttpGet("online-status")]
    public async Task<CompatVatprcStatusDto> Status()
    {
        var vatsimData = await VatsimService.GetOnlineData();
        var atcSchedule = await database.AtcBooking
            .Where(s => s.StartAt >= DateTimeOffset.UtcNow)
            .Include(s => s.User)
            .ToListAsync();
        return new CompatVatprcStatusDto
        {
            LastUpdated = vatsimData.General.UpdateTimestamp,
            Pilots = vatsimData.Pilots
                .Where(x =>
                    (x.FlightPlan?.Departure != null && vatprcAirportRegexp().IsMatch(x.FlightPlan?.Departure!)) ||
                    (x.FlightPlan?.Arrival != null && vatprcAirportRegexp().IsMatch(x.FlightPlan?.Arrival!)))
                .Select(x => new CompatPilotDto
                {
                    Cid = Convert.ToInt32(x.Cid),
                    Name = x.Name,
                    Callsign = x.Callsign,
                    Departure = x.FlightPlan?.Departure,
                    Arrival = x.FlightPlan?.Arrival,
                    Aircraft = x.FlightPlan?.AircraftShort,
                }),
            Controllers = vatsimData.Controllers
                .Where(x => vatprcControllerRegexp().IsMatch(x.Callsign))
                .Where(x => x.Facility > 0)
                .Select(x => new CompatControllerDto
                {
                    Cid = Convert.ToInt32(x.Cid),
                    Name = x.Name,
                    Callsign = x.Callsign,
                    Frequency = x.Frequency,
                }),
            FutureControllers = atcSchedule.Select(x => new CompatFutureControllerDto
            {
                Name = x.User!.FullName,
                Callsign = x.Callsign,
                Start = x.StartAt.ToUniversalTime().ToString("dd HH:mm"),
                StartUtc = x.StartAt.ToUniversalTime(),
                End = x.EndAt.ToUniversalTime().ToString("dd HH:mm"),
                EndUtc = x.EndAt.ToUniversalTime(),
            }),
        };
    }

    protected async Task<IActionResult> GetMetar(string icao)
    {
        var normalizedIcao = icao.ToUpperInvariant();
        var metar = await MetarService.GetMetar(normalizedIcao);
        if (string.IsNullOrEmpty(metar))
        {
            var content = Content($"{normalizedIcao} NO METAR", "text/plain", System.Text.Encoding.UTF8);
            content.StatusCode = (int)HttpStatusCode.NotFound;
            return content;
        }
        return Content(metar, "text/plain", System.Text.Encoding.UTF8);
    }

    [HttpGet("euroscope/metar/{icao}")]
    public async Task<IActionResult> GetMetarText(string icao)
    {
        return await GetMetar(icao);
    }

    [HttpGet("euroscope/metar/metar.php")]
    public async Task<IActionResult> GetMetarText2([FromQuery] string id)
    {
        return await GetMetar(id);
    }

    [HttpGet("homepage/events/vatsim")]
    public async Task<IActionResult> GetVatsimEvents()
    {
        var events = await VatsimService.GetDivisionEventsAsString();
        return Content(events, "application/json", System.Text.Encoding.UTF8);
    }

    [HttpGet("trackaudio/mandatory_version")]
    public async Task<IActionResult> GetTrackAudioVersion()
    {
        var lastVersion = await TrackAudioService.GetLastVersionAsync();
        return Content(lastVersion, "text/plain", System.Text.Encoding.UTF8);
    }

    [HttpGet("vplaaf/areas.json")]
    public async Task<IActionResult> GetVplaafAreas()
    {
        var areas = await vplaafAdapter.GetAreasAsync();
        return Content(areas, "application/json", System.Text.Encoding.UTF8);
    }
}
