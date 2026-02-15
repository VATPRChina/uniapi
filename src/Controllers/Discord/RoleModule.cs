using Discord;
using Discord.Interactions;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers.Discord;

public class RoleModule(
    VatsimAdapter vatsimAdapter,
    DatabaseAdapter db,
    ILogger<RoleModule> Logger) : InteractionModuleBase
{
    [CommandContextType([InteractionContextType.Guild, InteractionContextType.PrivateChannel])]
    [DefaultMemberPermissions(GuildPermission.Administrator)]
    [SlashCommand("role", "Get VATPRC role information for a user")]
    public async Task GetRoleAsync(IUser discordUser)
    {
        Logger.LogInformation("Role information requested for user {DiscordUserId}", discordUser.Id);

        var cid = await vatsimAdapter.GetCidByDiscordUserId(discordUser.Id.ToString());
        Logger.LogInformation("Got CID {Cid}", cid);

        if (cid == null)
        {
            Logger.LogInformation("No CID linked for Discord user {DiscordUserId}", discordUser.Id);
            var components = new ComponentBuilder()
                    .WithButton(label: "Link VATSIM Account", style: ButtonStyle.Link, url: "https://community.vatsim.net/")
                    .Build();
            await RespondAsync($"No VATSIM CID linked to Discord user <@{discordUser.Id}>.",
                components: components);
            return;
        }

        var user = await db.User.SingleOrDefaultAsync(u => u.Cid == cid);
        if (user == null)
        {
            await RespondAsync($"No user found for VATSIM CID {cid} for Discord user <@{discordUser.Id}>.");
            return;
        }

        var roles = UserRoleService.GetRoleClosure(user.Roles);

        await RespondAsync($"""
            <@{discordUser.Id}>'s roles in VATPRC:
            {string.Join("\n", roles.Select(r => $"- {r}"))}

            Assigned roles:
            {string.Join("\n", user.Roles.Select(r => $"- {r}"))}
            """);
    }
}
