using Arinc424;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

public static class Arinc424Helper
{
    extension(Record424 record)
    {
        public string RecordId => $"{record.Date}/{record.Code}/{record.Number}";
    }
}
