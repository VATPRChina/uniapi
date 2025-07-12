using Flurl.Http;

namespace Net.Vatprc.Uniapi.Utils;

public static class Utils
{
    public static readonly string UniapiUserAgent = $"Flurl.Http/{typeof(FlurlClient).Assembly.GetName().Version} {System.Reflection.Assembly.GetExecutingAssembly().GetName().Name}/{System.Reflection.Assembly.GetExecutingAssembly().GetName().Version}";
    public static readonly string UniapiUserAgentWs = $"Websocket.Client/{typeof(Websocket.Client.WebsocketClient).Assembly.GetName().Version} {System.Reflection.Assembly.GetExecutingAssembly().GetName().Name}/{System.Reflection.Assembly.GetExecutingAssembly().GetName().Version}";

    public static string StripPrefix(this string value, string prefix)
    {
        return value.StartsWith(prefix, StringComparison.InvariantCultureIgnoreCase) ? value[prefix.Length..] : value;
    }
}
