namespace Net.Vatprc.Uniapi.Adapters.EmailAdapter;

public abstract class EmailBase
{
    public abstract string GetSubject();

    public abstract string GetHtml();

    public abstract string GetPlainText();
}
