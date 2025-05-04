namespace Net.Vatprc.Uniapi.Utils;

public static class DateTimeUtil
{
    public static DateTimeOffset RoundUp(this DateTimeOffset dt, TimeSpan d)
    {
        return new DateTimeOffset((dt.Ticks + d.Ticks - 1) / d.Ticks * d.Ticks, dt.Offset);
    }
}
