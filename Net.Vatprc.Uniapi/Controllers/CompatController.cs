using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Authorization;
using System.Text.RegularExpressions;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Compatibility with existing services.
/// </summary>
[ApiController, Route("api/compat")]
public partial class CompatController(VatsimService VatsimService) : ControllerBase
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
        public required string End { get; set; }
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
    [GeneratedRegex("Z[BMSPGJYWLH][A-Z]{2}")]
    protected static partial Regex vatprcAirportRegexp();

    [HttpGet("online-status")]
    [AllowAnonymous]
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
                .Where(x => vatprcAirportRegexp().IsMatch(x.Callsign))
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
                End = x.Finish.ToUniversalTime().ToString("dd HH:mm"),
            }),
        };
    }
}
