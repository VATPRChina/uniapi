using Discord;
using Discord.Interactions;

namespace Net.Vatprc.Uniapi.Controllers.Discord;

public class MessageModule : InteractionModuleBase
{
    protected const string MESSAGE_MODAL_ID = "edit_message_modal";

    [CommandContextType([InteractionContextType.Guild, InteractionContextType.PrivateChannel])]
    [DefaultMemberPermissions(GuildPermission.ManageMessages)]
    [SlashCommand("post", "Post message in current channel")]
    public async Task SendMessage()
    {
        await RespondWithModalAsync<EditMessageModal>(MESSAGE_MODAL_ID, modifyModal: modal =>
        {
            modal.UpdateTextInput("channel_id", Context.Channel.Id.ToString());
        });
    }

    [CommandContextType([InteractionContextType.Guild, InteractionContextType.PrivateChannel])]
    [DefaultMemberPermissions(GuildPermission.ManageMessages)]
    [MessageCommand("Edit Message")]
    public async Task EditMessage(IMessage message)
    {
        if (message is not IUserMessage userMessage || message.Author.Id != Context.Client.CurrentUser.Id)
        {
            await RespondAsync($"Message id = {message.Id} not found or not editable", ephemeral: true);
        }

        await RespondWithModalAsync<EditMessageModal>(MESSAGE_MODAL_ID, modifyModal: modal =>
        {
            modal.UpdateTextInput("channel_id", Context.Channel.Id.ToString());
            modal.UpdateTextInput("message_id", message.Id.ToString());
            modal.UpdateTextInput("message", message.Content);
        });
    }

    public class EditMessageModal : IModal
    {
        public string Title => "Create or edit a message";

        [InputLabel("Channel ID")]
        [ModalTextInput("channel_id", TextInputStyle.Short, "Channel ID")]
        public string ChannelId { get; set; } = string.Empty;

        [RequiredInput(false)]
        [InputLabel("Message ID")]
        [ModalTextInput("message_id", TextInputStyle.Short, "Message ID (leave empty to post new)")]
        public string MessageId { get; set; } = string.Empty;

        [InputLabel("Message")]
        [ModalTextInput("message", TextInputStyle.Paragraph)]
        public string Message { get; set; } = string.Empty;

        [RequiredInput(false)]
        [InputLabel("Links")]
        [ModalTextInput("links", TextInputStyle.Paragraph, "Links (one per line in format 'Label|URL' or just 'URL')")]
        public string Links { get; set; } = string.Empty;
    }

    [ModalInteraction(MESSAGE_MODAL_ID)]
    public async Task ModalResponse(EditMessageModal modal)
    {
        var channelId = ulong.Parse(modal.ChannelId);
        if (await Context.Client.GetChannelAsync(channelId) is not IMessageChannel channel)
        {
            await RespondAsync($"Channel id = {channelId} not found", ephemeral: true);
            return;
        }

        ComponentBuilder? builder = null;
        if (!string.IsNullOrEmpty(modal.Links))
        {
            var links = modal.Links.Split('\n', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
            builder = new ComponentBuilder();
            foreach (var link in links)
            {
                var parsed = link.Split('|');
                if (parsed.Length >= 2)
                {
                    builder.WithButton(label: parsed[0], style: ButtonStyle.Link, url: parsed[1]);
                }
                else
                {
                    builder.WithButton(label: "Link", style: ButtonStyle.Link, url: parsed[0]);
                }
            }
        }

        if (ulong.TryParse(modal.MessageId, out var messageId)
            && await channel.GetMessageAsync(messageId) is IUserMessage message)
        {
            await message.ModifyAsync(msg =>
            {
                msg.Content = modal.Message;
                msg.Components = builder?.Build();
            });
        }
        else
        {
            if (string.IsNullOrEmpty(modal.MessageId))
            {
                await channel.SendMessageAsync(modal.Message, components: builder?.Build());
            }
            else
            {
                await RespondAsync($"Message id = {messageId} not found in channel id = {channel.Id}", ephemeral: true);
                return;
            }
        }

        await RespondAsync("Message send/updated.", ephemeral: true);
    }
}
