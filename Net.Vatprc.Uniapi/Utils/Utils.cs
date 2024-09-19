using Flurl.Http;

namespace Net.Vatprc.Uniapi.Utils;

public static class Utils
{
    public static readonly string UniapiUserAgent = $"Flurl.Http/{typeof(FlurlClient).Assembly.GetName().Version} {System.Reflection.Assembly.GetExecutingAssembly().GetName().Name}/{System.Reflection.Assembly.GetExecutingAssembly().GetName().Version}";
}
