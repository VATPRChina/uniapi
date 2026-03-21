using Discord;
using Discord.Interactions;

namespace Net.Vatprc.Uniapi.Controllers.Discord;

public class RoleExporterModule : InteractionModuleBase
{
    [CommandContextType([InteractionContextType.Guild, InteractionContextType.PrivateChannel])]
    [DefaultMemberPermissions(GuildPermission.ManageRoles)]
    [SlashCommand("export-roles", "Export all roles in current server as JSON")]
    public async Task SendMessage()
    {
        var roles = Context.Guild.Roles.Select(r => new
        {
            r.Id,
            r.Name,
            r.Color,
            r.Position,
            r.Permissions,
            r.IsHoisted,
            r.IsMentionable,
        });

        var json = System.Text.Json.JsonSerializer.Serialize(roles, new System.Text.Json.JsonSerializerOptions
        {
            WriteIndented = true,
        });

        await RespondWithFileAsync(new FileAttachment(new MemoryStream(System.Text.Encoding.UTF8.GetBytes(json)), "roles.json"), $"JSON", ephemeral: true);
    }
}
