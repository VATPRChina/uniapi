using System.Text.Json.Serialization;
using Discord.Interactions;
using Flurl.Http;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers.Discord;

public class MetarModule(RudiMetarService MetarService) : InteractionModuleBase
{
    [SlashCommand("metar", "Get METAR for an airport")]
    public async Task WhoAmIAsync(string icao)
    {
        if (icao.Length != 4)
        {
            await RespondAsync($"ICAO code must be 4 characters long, find {icao}");
            return;
        }
        var result = await MetarService.GetMetar(icao);
        if (string.IsNullOrWhiteSpace(result))
        {
            await RespondAsync($"No METAR found for airport {icao}");
            return;
        }
        await RespondAsync(result);
    }
}
