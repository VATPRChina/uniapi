using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// Partial for Runway (PG) record in ARINC 424 Section 4.1.10.1.
/// </summary>
public class Runway
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    /// <summary>
    /// Airport/Heliport Identifier. (Section 5.6) The four-character ICAO Location
    /// Identifier, or three or four-character Domestic Identifier.
    /// </summary>
    public string? AirportIdentifier => Airport?.Identifier;

    public Ulid AirportId { get; set; }
    public Airport? Airport { get; set; }

    /// <summary>
    /// Runway Identifier. (Section 5.46)
    /// </summary>
    public string Identifier { get; set; } = string.Empty;

    /// <summary>
    /// Landing Threshold Latitude. (Section 5.36)
    /// </summary>
    public double Latitude { get; set; }

    /// <summary>
    /// Landing Threshold Longitude. (Section 5.37)
    /// </summary>
    public double Longitude { get; set; }

    public class Configuration : IEntityTypeConfiguration<Runway>
    {
        public void Configure(EntityTypeBuilder<Runway> builder)
        {
            builder.ToTable("runway", "navdata");

            builder.HasOne(x => x.Airport)
                .WithMany(x => x.Runways)
                .HasForeignKey(x => x.AirportId)
                .IsRequired();
        }
    }
}
