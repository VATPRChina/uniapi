using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Adapters.EmailAdapter;

public class TrainingApplicationResponseEmail : MjmlEmailBase
{
    protected readonly TrainingApplication application;
    protected readonly TrainingApplicationResponse response;

    public TrainingApplicationResponseEmail(TrainingApplication application, TrainingApplicationResponse response)
    {
        this.application = application;
        this.response = response;
    }

    public override string GetActionText()
    {
        return "View all training requests";
    }

    public override string GetActionUrl()
    {
        return "/controllers/trainings";
    }

    public override string GetEmailReasonText()
    {
        return "you have filed a training application";
    }

    public override string GetPostActionMjml()
    {
        return $$"""
            <mj-divider border-color="#dee2e6" border-width="2px"></mj-divider>
            <mj-text font-size="18px" font-weight="bold">Response Details</mj-text>
            <mj-text>Student: {{application.Trainee!.FullName}}/{{application.Trainee!.Cid}}</mj-text>
            <mj-text>Title: {{application.Name}}</mj-text>
            <mj-text>Response: {{(response.SlotId != null ? "Approved" : "Rejected")}}</mj-text>
            <mj-text>Comments: {{response.Comment}}</mj-text>
            """;
    }

    public override string GetPostActionText()
    {
        return $$"""

            Response Details
            Student: {{application.Trainee!.FullName}}/{{application.Trainee!.Cid}}
            Title: {{application.Name}}
            Response: {{(response.SlotId != null ? "Approved" : "Rejected")}}
            Comments: {{response.Comment}}
            """;
    }

    public override string GetPreActionMjml()
    {
        return $$"""
            <mj-text>We have received a new response for your training request.</mj-text>
            <mj-text>You can view the training request details in the training request list.</mj-text>
            """;
    }

    public override string GetPreActionText()
    {
        return "We have received a new response for your training request. You can view the training request details in the training request list.";
    }

    public override string GetPreviewText()
    {
        return "New response for your training request";
    }

    public override string GetSubject()
    {
        return "New response to training request";
    }

    public override string GetTitleText()
    {
        return "New response for your training request";
    }
}
