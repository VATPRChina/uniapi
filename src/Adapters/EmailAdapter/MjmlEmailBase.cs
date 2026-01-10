using Mjml.Net;

namespace Net.Vatprc.Uniapi.Adapters.EmailAdapter;

public abstract class MjmlEmailBase : EmailBase
{
    public abstract string GetPreActionMjml();

    public abstract string GetPreActionText();

    public abstract string GetActionText();

    public abstract string GetActionUrl();

    public abstract string GetPostActionMjml();

    public abstract string GetPostActionText();

    public abstract string GetPreviewText();

    public abstract string GetTitleText();

    public abstract string GetEmailReasonText();

    public override string GetHtml()
    {
        var mjmlRenderer = new MjmlRenderer();

        var text = $$"""
<mjml>
  <mj-head>
    <mj-font name="Outfit"
      href="https://fonts.loli.net/css2?family=Outfit:wght@100..900" />
    <mj-attributes>
      <mj-all font-family="Outfit,ui-sans-serif,system-ui,sans-serif,Apple Color Emoji,Segoe UI Emoji,Segoe UI Symbol,Noto Color Emoji" font-size="12pt" line-height="1.5" />
    </mj-attributes>
    <mj-style>
      a {
      	color: inherit;
      }
    </mj-style>
    <mj-preview>{{GetPreviewText()}}</mj-preview>
    <mj-title>{{GetTitleText()}}</mj-title>
  </mj-head>
  <mj-body background-color="#f9fafb">
    <mj-section>
      <mj-column>
        <mj-image height="32px" width="85.1px" src="https://cdn.sa.net/2026/01/10/VrnqAC4oJTfBkG2.png" align="left"></mj-image>
      </mj-column>
    </mj-section>
    <mj-section background-color="#FFF">
      <mj-column>
        <mj-text font-size="24px" font-weight="bold">{{GetTitleText()}}</mj-text>
        {{GetPreActionMjml()}}
        <mj-button color="#FFF" background-color="#AB1615" href="https://www.vatprc.net{{GetActionUrl()}}">{{GetActionText()}}</mj-button>
        {{GetPostActionMjml()}}
      </mj-column>
    </mj-section>
    <mj-section>
      <mj-column>
        <mj-text color="#868e96">You are receiving this email because {{GetEmailReasonText()}} in VATPRC.</mj-text>
        <mj-text color="#868e96">This is an notification email which you cannot unsubscribe from. To manage other email communication preferences, visit your <a href="https://www.vatprc.net/users/me">account settings</a>. For further questions, please contact <a href="mailto:feedback@vatprc.net">VATPRC Support</a>.</mj-text>
        <mj-text></mj-text>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
""";

        var options = new MjmlOptions
        {
            Beautify = false
        };

        var (html, errors) = mjmlRenderer.Render(text, options);

        return html;
    }

    public override string GetPlainText()
    {
        return $$"""
{{GetTitleText()}}
---

{{GetPreActionText()}}

{{GetActionText()}}: https://www.vatprc.net{{GetActionUrl()}}

{{GetPostActionText()}}

---
You are receiving this email because {{GetEmailReasonText()}} in VATPRC.
This is an notification email which you cannot unsubscribe from.
To manage other email communication preferences, visit your account settings https://www.vatprc.net/users/me.
For further questions, please contact VATPRC Support feedback@vatprc.net.
""";
    }
}
