using Discord.Interactions;
using Net.Vatprc.Uniapi.Adapters;

namespace Net.Vatprc.Uniapi.Controllers.Discord;

public class MetarModule(MetarAdapter MetarService, ILogger<MetarModule> Logger) : InteractionModuleBase
{
    [SlashCommand("metar", "Get METAR for an airport")]
    public async Task WhoAmIAsync(string icao)
    {
        Logger.LogInformation($"METAR requested for {icao}");
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
