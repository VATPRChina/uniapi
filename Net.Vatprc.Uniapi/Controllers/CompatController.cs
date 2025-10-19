using System.Net;
using System.Text.RegularExpressions;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Compatibility with existing services.
/// </summary>
[ApiController, Route("api/compat")]
[AllowAnonymous]
public partial class CompatController(
    VatsimAdapter VatsimService,
    MetarAdapter MetarService,
    TrackAudioAdapter TrackAudioService) : ControllerBase
{
    public class ControllerDto
    {
        public required int Cid { get; set; }
        public required string Name { get; set; }
        public required string Callsign { get; set; }
        public required string Frequency { get; set; }
    }

    public class FutureControllerDto
    {
        public required string Callsign { get; set; }
        public required string Name { get; set; }
        public required string Start { get; set; }
        public required DateTimeOffset StartUtc { get; set; }
        public required string End { get; set; }
        public required DateTimeOffset EndUtc { get; set; }
    }

    public class PilotDto
    {
        public required int Cid { get; set; }
        public required string Name { get; set; }
        public required string Callsign { get; set; }
        public required string? Departure { get; set; }
        public required string? Arrival { get; set; }
        public required string? Aircraft { get; set; }
    }

    public class VatprcStatusDto
    {
        public required DateTimeOffset LastUpdated { get; set; }
        public required IEnumerable<PilotDto> Pilots { get; set; }
        public required IEnumerable<ControllerDto> Controllers { get; set; }
        public required IEnumerable<FutureControllerDto> FutureControllers { get; set; }
    }


    [GeneratedRegex("^(Z[BSGUHWJPLYM][A-Z0-9]{2}(_[A-Z0-9]*)?_(DEL|GND|TWR|APP|DEP|CTR))|(PRC_FSS)$")]
    protected static partial Regex vatprcControllerRegexp();
    [GeneratedRegex("^Z[BMSPGJYWLH][A-Z]{2}")]
    protected static partial Regex vatprcAirportRegexp();

    [HttpGet("online-status")]
    public async Task<VatprcStatusDto> Status()
    {
        var vatsimData = await VatsimService.GetOnlineData();
        var atcSchedule = await VatsimService.GetAtcSchedule();
        return new VatprcStatusDto
        {
            LastUpdated = vatsimData.General.UpdateTimestamp,
            Pilots = vatsimData.Pilots
                .Where(x =>
                    (x.FlightPlan?.Departure != null && vatprcAirportRegexp().IsMatch(x.FlightPlan?.Departure!)) ||
                    (x.FlightPlan?.Arrival != null && vatprcAirportRegexp().IsMatch(x.FlightPlan?.Arrival!)))
                .Select(x => new PilotDto
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
                .Select(x => new ControllerDto
                {
                    Cid = Convert.ToInt32(x.Cid),
                    Name = x.Name,
                    Callsign = x.Callsign,
                    Frequency = x.Frequency,
                }),
            FutureControllers = atcSchedule.Select(x => new FutureControllerDto
            {
                Name = $"{x.User.FirstName} {x.User.LastName}",
                Callsign = x.Callsign,
                Start = x.Start.ToUniversalTime().ToString("dd HH:mm"),
                StartUtc = x.Start.ToUniversalTime(),
                End = x.Finish.ToUniversalTime().ToString("dd HH:mm"),
                EndUtc = x.Finish.ToUniversalTime(),
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
}
