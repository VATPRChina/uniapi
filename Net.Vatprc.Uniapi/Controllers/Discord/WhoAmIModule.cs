using System.Text.Json.Serialization;
using Discord.Interactions;
using Flurl;
using Flurl.Http;

namespace Net.Vatprc.Uniapi.Controllers.Discord;

public class WhoAmIModule : InteractionModuleBase
{
    [SlashCommand("whoami", "Check VATSIM identity")]
    public async Task WhoAmIAsync()
    {
        var result = await $"https://api.vatsim.net/v2/members/discord/{Context.User.Id}"
            .GetJsonAsync<DiscordResult>();
        await RespondAsync($"You are {result.Id}/{result.UserId}");
    }

    public record DiscordResult
    {
        [JsonPropertyName("id")]
        public required string Id { get; set; }

        [JsonPropertyName("user_id")]
        public required string UserId { get; set; }
    }
}
