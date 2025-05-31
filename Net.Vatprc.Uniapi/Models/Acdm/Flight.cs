using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Acdm;

public class Flight
{
    public Ulid Id { get; set; }

    public DateTimeOffset? FinalizedAt { get; set; }

    public string Cid { get; set; } = string.Empty;

    public string Callsign { get; set; } = string.Empty;

    public DateTimeOffset LastObservedAt { get; set; }

    public string Aircraft { get; set; } = string.Empty;

    public string Equipment { get; set; } = string.Empty;

    public string Transponder { get; set; } = string.Empty;

    public string NavigationPerformance { get; set; } = string.Empty;

    public double Latitude { get; set; }

    public double Longitude { get; set; }

    public long Altitude { get; set; }

    public string Departure { get; set; } = string.Empty;

    public string? DepartureGate { get; set; }

    public string? DepartureRunway { get; set; }

    public string Arrival { get; set; } = string.Empty;

    public string? ArrivalGate { get; set; }

    public string? ArrivalRunway { get; set; }

    public uint CruiseTas { get; set; }

    public string RawRoute { get; set; } = string.Empty;

    public FlightState State { get; set; }

    public enum FlightState
    {
        SCHEDULED,
        PRE_DEPARTURE,
        DEPARTURE_TAXI,
        CRUISE,
        ARRIVAL_TAXI,
        FINALIZATION,
        UNKNOWN,
    }

    public bool SupportRvsm => Equipment.Contains('W');
    public bool SupportRnav1 => SupportRnav1Equipment && SupportRnav1Pbn;
    public bool SupportRnav1Equipment => Equipment.Contains('R');
    public bool SupportRnav1Pbn => NavigationPerformance.Contains("D1") || NavigationPerformance.Contains("D2");
    public bool SupportRnpArWithRf => NavigationPerformance.Contains("T1");
    public bool SupportRnpArWithoutRf => NavigationPerformance.Contains("T2");
    public bool HasTransponder => !string.IsNullOrEmpty(Transponder);

    public class Configuration : IEntityTypeConfiguration<Flight>
    {
        public void Configure(EntityTypeBuilder<Flight> builder)
        {
            builder.Property(x => x.FinalizedAt)
                .IsRequired(false)
                .HasDefaultValue(null);

            builder.HasIndex(x => x.Callsign);

            builder.HasIndex(x => x.Cid);

            builder.Property(x => x.DepartureGate)
                .IsRequired(false);

            builder.Property(x => x.ArrivalGate)
                .IsRequired(false);

            builder.Property(x => x.State)
                .HasConversion<string>();
        }
    }
}
