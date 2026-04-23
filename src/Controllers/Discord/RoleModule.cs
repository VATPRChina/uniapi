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
    private const string APPLY_BUTTON_ID_PREFIX = "role:apply";

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
            var linkComponents = new ComponentBuilder()
                    .WithButton(label: "Link VATSIM Account", style: ButtonStyle.Link, url: "https://community.vatsim.net/")
                    .Build();
            await RespondAsync($"No VATSIM CID linked to Discord user <@{discordUser.Id}>.",
                components: linkComponents);
            return;
        }

        if (Context.Guild == null)
        {
            await RespondAsync("This command can only inspect and apply Discord roles inside a server.", ephemeral: true);
            return;
        }

        var user = await db.User.SingleOrDefaultAsync(u => u.Cid == cid);
        if (user == null)
        {
            await RespondAsync($"No user found for VATSIM CID {cid} for Discord user <@{discordUser.Id}>.");
            return;
        }

        var guildUser = await Context.Guild.GetUserAsync(discordUser.Id);
        if (guildUser == null)
        {
            await RespondAsync($"Discord user <@{discordUser.Id}> is not a member of this server.", ephemeral: true);
            return;
        }

        var plan = await BuildRoleSyncPlanAsync(discordUser, guildUser, user);
        var components = new ComponentBuilder();
        if (plan.RolesToAdd.Count > 0 || plan.RolesToRemove.Count > 0)
        {
            components.WithButton(
                label: "Apply",
                style: ButtonStyle.Primary,
                customId: $"{APPLY_BUTTON_ID_PREFIX}:{discordUser.Id}");
        }

        await RespondAsync(
            BuildRoleSummary(plan),
            components: components.ActionRows.Count > 0 ? components.Build() : null,
            allowedMentions: AllowedMentions.None);
    }

    [ComponentInteraction($"{APPLY_BUTTON_ID_PREFIX}:*")]
    public async Task ApplyRoleAsync(string discordUserId)
    {
        if (Context.Guild == null)
        {
            await RespondAsync("This action can only be used inside a server.", ephemeral: true);
            return;
        }

        if (!ulong.TryParse(discordUserId, out var targetDiscordUserId))
        {
            await RespondAsync("Invalid target user for role sync.", ephemeral: true);
            return;
        }

        logger.LogInformation("Role sync requested for user {DiscordUserId} by {RequesterDiscordUserId}",
            targetDiscordUserId, Context.User.Id);

        var cid = await vatsimAdapter.GetCidByDiscordUserId(targetDiscordUserId.ToString());
        if (cid == null)
        {
            await RespondAsync($"No VATSIM CID linked to Discord user <@{targetDiscordUserId}>.", ephemeral: true);
            return;
        }

        var user = await db.User.SingleOrDefaultAsync(u => u.Cid == cid);
        if (user == null)
        {
            await RespondAsync($"No user found for VATSIM CID {cid} for Discord user <@{targetDiscordUserId}>.", ephemeral: true);
            return;
        }

        var guildUser = await Context.Guild.GetUserAsync(targetDiscordUserId);
        if (guildUser == null)
        {
            await RespondAsync($"Discord user <@{targetDiscordUserId}> is not a member of this server.", ephemeral: true);
            return;
        }

        var plan = await BuildRoleSyncPlanAsync(guildUser, guildUser, user);
        if (plan.RolesToAdd.Count > 0)
        {
            await guildUser.AddRolesAsync(plan.RolesToAdd);
        }

        if (plan.RolesToRemove.Count > 0)
        {
            await guildUser.RemoveRolesAsync(plan.RolesToRemove);
        }

        logger.LogInformation(
            "Applied role sync for user {DiscordUserId}: added {AddedRoleCount}, removed {RemovedRoleCount}",
            targetDiscordUserId,
            plan.RolesToAdd.Count,
            plan.RolesToRemove.Count);

        await RespondAsync($"""
            Applied role sync for <@{targetDiscordUserId}>.
            Added: {(plan.RolesToAdd.Count == 0 ? "none" : string.Join(", ", plan.RolesToAdd.Select(roleId => $"<@&{roleId}>")))}
            Removed: {(plan.RolesToRemove.Count == 0 ? "none" : string.Join(", ", plan.RolesToRemove.Select(roleId => $"<@&{roleId}>")))}
            """, ephemeral: true, allowedMentions: AllowedMentions.None);
    }

    private async Task<RoleSyncPlan> BuildRoleSyncPlanAsync(IUser discordUser, IGuildUser guildUser, Models.User user)
    {
        var vatsimRoles = UserRoleService.GetRoleClosure(user.Roles).ToHashSet();
        var expectedRoles = (await roleMapper.GetUserRoles(user)).ToHashSet();
        var managedRoles = roleMapper.GetAllManagedRoles().ToHashSet();
        var currentRoles = guildUser.RoleIds.ToHashSet();

        var rolesToAdd = expectedRoles
            .Where(roleId => !currentRoles.Contains(roleId))
            .OrderBy(roleId => roleId)
            .ToArray();

        var rolesToRemove = currentRoles
            .Where(roleId => managedRoles.Contains(roleId) && !expectedRoles.Contains(roleId))
            .OrderBy(roleId => roleId)
            .ToArray();

        return new RoleSyncPlan(
            discordUser,
            user.Roles.ToHashSet(),
            vatsimRoles.OrderBy(role => role).ToArray(),
            managedRoles,
            currentRoles.OrderBy(roleId => roleId).ToArray(),
            expectedRoles,
            rolesToAdd,
            rolesToRemove);
    }

    private static string BuildRoleSummary(RoleSyncPlan plan)
    {
        var discordRoleLines = plan.CurrentRoles.Select(roleId =>
        {
            if (!plan.ManagedRoles.Contains(roleId))
            {
                return $"- ⚠️ <@&{roleId}> (Unmanaged)";
            }

            return plan.ExpectedRoles.Contains(roleId)
                ? $"- ✅ <@&{roleId}> (As expected)"
                : $"- 🚫 <@&{roleId}> (Should remove)";
        });

        var missingRoleLines = plan.RolesToAdd
            .Select(roleId => $"- ➕ <@&{roleId}> (Should add)");

        return $"""
            <@{plan.DiscordUser.Id}>'s roles in VATPRC:
            {string.Join("\n", plan.VatsimRoles.Select(role => $"- {role} {(plan.DirectRoles.Contains(role) ? "" : "(Inherited)")}"))}

            Discord roles:
            {string.Join("\n", discordRoleLines.Concat(missingRoleLines))}
            """;
    }

    private record RoleSyncPlan(
        IUser DiscordUser,
        HashSet<string> DirectRoles,
        IReadOnlyCollection<string> VatsimRoles,
        HashSet<ulong> ManagedRoles,
        IReadOnlyCollection<ulong> CurrentRoles,
        HashSet<ulong> ExpectedRoles,
        IReadOnlyCollection<ulong> RolesToAdd,
        IReadOnlyCollection<ulong> RolesToRemove);
}
