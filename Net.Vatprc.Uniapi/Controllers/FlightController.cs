using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Utils;
using System.Diagnostics.CodeAnalysis;
using Microsoft.AspNetCore.Authorization;
using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Flight information.
/// </summary>
[ApiController, Route("api/flights")]
public class FlightController(VATPRCContext DbContext) : ControllerBase
{
    public record FlightDto
    {
        public Ulid Id { get; init; }
        public string Cid { get; init; }
        public string Callsign { get; init; }
        public DateTimeOffset LastObservedAt { get; init; }
        public string Departure { get; init; }
        public string Arrival { get; init; }

        public FlightDto(Flight flight)
        {
            Id = flight.Id;
            Cid = flight.Cid;
            Callsign = flight.Callsign;
            LastObservedAt = flight.LastObservedAt;
            Departure = flight.Departure;
            Arrival = flight.Arrival;
        }
    }

    [HttpGet("by-callsign/{callsign}")]
    [AllowAnonymous]
    public async Task<FlightDto> GetByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        return new FlightDto(flight);
    }

    public record WarningMessage
    {
        public string MessageCode { get; init; } = string.Empty;
    }

    [HttpGet("by-callsign/{callsign}/warnings")]
    [AllowAnonymous]
    public async Task<IEnumerable<WarningMessage>> GetWarningByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        var result = new List<WarningMessage>();
        if (!flight.Equipment.Contains("W"))
        {
            result.Add(new WarningMessage { MessageCode = "no_rvsm" });
        }
        if (!flight.Equipment.Contains("R")
            || (!flight.NavigationPerformance.Contains("D1") &&
                !flight.NavigationPerformance.Contains("D2")))
        {
            result.Add(new WarningMessage { MessageCode = "no_rnav1" });
        }
        if (flight.NavigationPerformance.Contains("T1"))
        {
            result.Add(new WarningMessage { MessageCode = "rnp_ar" });
        }
        if (flight.NavigationPerformance.Contains("T2"))
        {
            result.Add(new WarningMessage { MessageCode = "rnp_ar_without_rf" });
        }
        if (string.IsNullOrEmpty(flight.Transponder))
        {
            result.Add(new WarningMessage { MessageCode = "no_transponder" });
        }
        return result;
    }
}
