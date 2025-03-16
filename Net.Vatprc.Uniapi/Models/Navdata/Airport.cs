using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// Partial for Airport (PA) record in ARINC 424 Section 4.1.7.1.
/// </summary>
public class Airport
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    /// <summary>
    /// Airport/Heliport Identifier. (Section 5.6) The four-character ICAO Location
    /// Identifier, or three or four-character Domestic Identifier.
    /// </summary>
    public string Identifier { get; set; } = string.Empty;

    /// <summary>
    /// Latitude. (Section 5.36)
    /// </summary>
    public double Latitude { get; set; }

    /// <summary>
    /// Longitude. (Section 5.37)
    /// </summary>
    public double Longitude { get; set; }

    /// <summary>
    /// Airport/Heliport Elevation. (Section 5.55) In feet.
    /// </summary>
    public int Elevation { get; set; }

    public IList<AirportGate> Gates { get; set; } = [];
    public IList<Runway> Runways { get; set; } = [];
    public IList<Procedure> Procedures { get; set; } = [];

    public class Configuration : IEntityTypeConfiguration<Airport>
    {
        public void Configure(EntityTypeBuilder<Airport> builder)
        {
            builder.ToTable("airport", "navdata");

            builder.HasIndex(x => x.Identifier)
                .IsUnique();
        }
    }
}
