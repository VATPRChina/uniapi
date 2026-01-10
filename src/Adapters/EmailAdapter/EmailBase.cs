namespace Net.Vatprc.Uniapi.Adapters.EmailAdapter;

public abstract class EmailBase
{
    public IDictionary<string, string> Data { get; init; } = new Dictionary<string, string>();

    public abstract string GetSubject();

    public abstract string GetHtml();

    public abstract string GetPlainText();
}
