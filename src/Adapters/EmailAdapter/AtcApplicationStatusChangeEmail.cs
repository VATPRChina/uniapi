namespace Net.Vatprc.Uniapi.Adapters.EmailAdapter;

public class AtcApplicationStatusChangeEmail : NotificationEmail
{
    public AtcApplicationStatusChangeEmail(Ulid id, string status)
        : base($"Your ATC application status has been changed to {status}",
            string.Empty,
            $"https://www.vatprc.net/controllers/applications/{id}")
    {
    }
}
