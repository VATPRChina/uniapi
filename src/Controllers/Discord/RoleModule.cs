using Discord;
using Discord.Interactions;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers.Discord;

public class RoleModule(
    VatsimAdapter vatsimAdapter,
    DatabaseAdapter db,
    ILogger<RoleModule> logger,
    DiscordRoleMapper roleMapper) : InteractionModuleBase
{
    [CommandContextType([InteractionContextType.Guild, InteractionContextType.PrivateChannel])]
    [DefaultMemberPermissions(GuildPermission.Administrator)]
    [SlashCommand("role", "Get VATPRC role information for a user")]
    public async Task GetRoleAsync(IUser discordUser)
    {
        logger.LogInformation("Role information requested for user {DiscordUserId}", discordUser.Id);

        var cid = await vatsimAdapter.GetCidByDiscordUserId(discordUser.Id.ToString());
        logger.LogInformation("Got CID {Cid}", cid);

        if (cid == null)
        {
            logger.LogInformation("No CID linked for Discord user {DiscordUserId}", discordUser.Id);
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

        var currentRoles = (await Context.Guild.GetUserAsync(discordUser.Id)).RoleIds;
        var managedRoles = roleMapper.GetAllManagedRoles();

        await RespondAsync($"""
            <@{discordUser.Id}>'s roles in VATPRC:
            {string.Join("\n", roles.Select(r => $"- {r} {(user.Roles.Contains(r) ? "" : "(Inherited)")}"))}

            Discord roles:
            {string.Join("\n", currentRoles.Select(r => $"- {(managedRoles.Contains(r) ? currentRoles.Contains(r) ? "✅" : "❌" : "⚠️")} <@&{r}> {(managedRoles.Contains(r) ? currentRoles.Contains(r) ? "(As expected)" : "(Should add)" : "(Unmanaged)")}"))}
            {string.Join("\n", currentRoles.Where(r => !currentRoles.Any(er => er == r) && managedRoles.Contains(r)).Select(r => $"- 🚫 <@&{r}> (Should remove)"))}

            {string.Join(';', (await Context.Client.GetGuildsAsync()).Select(g => $"{g.Name} {g.Id}"))}
            """, allowedMentions: AllowedMentions.None);
    }
}
