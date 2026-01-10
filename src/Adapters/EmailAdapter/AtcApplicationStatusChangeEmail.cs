using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Adapters.EmailAdapter;

public class AtcApplicationStatusChangeEmail : MjmlEmailBase
{
    protected readonly AtcApplication application;

    public AtcApplicationStatusChangeEmail(AtcApplication application)
    {
        this.application = application;
    }

    public override string GetActionText()
    {
        return "View application";
    }

    public override string GetActionUrl()
    {
        return $"/controllers/applications/{application.Id}";
    }

    public override string GetEmailReasonText()
    {
        return "your have submitted an ATC application";
    }

    public override string GetPostActionMjml()
    {
        return string.Empty;
    }

    public override string GetPostActionText()
    {
        return string.Empty;
    }

    protected string GetStatusText()
    {
        return application.Status switch
        {
            AtcApplicationStatus.Submitted => "pending review",
            AtcApplicationStatus.InWaitlist => "in the waitlist queue",
            AtcApplicationStatus.Approved => "approved",
            AtcApplicationStatus.Rejected => "rejected",
            _ => "changed status"
        };
    }

    public override string GetPreActionMjml()
    {
        return $$"""
            <mj-text>Your ATC application is now {{GetStatusText()}}.</mj-text>
            <mj-text>You can view the details in the ATC application list.</mj-text>
            """;
    }

    public override string GetPreActionText()
    {
        return $"""
            Your ATC application is now {GetStatusText()}.
            You can view the details in the ATC application list.
            """;
    }

    public override string GetPreviewText()
    {
        return "Your ATC application status has changed.";
    }

    public override string GetSubject()
    {
        return "Your ATC application status has changed";
    }

    public override string GetTitleText()
    {
        return "ATC Application Status Update";
    }
}
