using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Acdm;

public class Flight
{
    public string Cid { get; set; } = string.Empty;

    public string Callsign { get; set; } = string.Empty;

    public DateTimeOffset LastObservedAt { get; set; }

    public double Latitude { get; set; }

    public double Longitude { get; set; }

    public long Altitude { get; set; }

    public string Departure { get; set; } = string.Empty;

    public string Arrival { get; set; } = string.Empty;

    public uint CruiseTas { get; set; }

    public string RawRoute { get; set; } = string.Empty;

    public FlightState State { get; set; }

    public enum FlightState
    {
        PRE_DEPARTURE,
        PUSHBACK,
        TAXI,
        TAKEOFF,
        CLIMB,
        CRUISE,
        DESCENT,
        APPROACH,
        LANDING,
        TAXI_ARRIVAL,
        SHUTDOWN,
    }

    public class Configuration : IEntityTypeConfiguration<Flight>
    {
        public void Configure(EntityTypeBuilder<Flight> builder)
        {
            builder.HasKey(x => x.Callsign);

            builder.HasIndex(x => x.Cid)
                .IsUnique();
        }
    }
}
