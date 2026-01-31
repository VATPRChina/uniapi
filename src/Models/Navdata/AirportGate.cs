using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// Partial for Airport Gate (PB) record in ARINC 424 Section 4.1.8.1.
/// </summary>
public class AirportGate
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
    /// Gate Identifier. (Section 5.56)
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

    public class Configuration : IEntityTypeConfiguration<AirportGate>
    {
        public void Configure(EntityTypeBuilder<AirportGate> builder)
        {
            builder.ToTable("airport_gate", "navdata");

            builder.Ignore(x => x.AirportIdentifier);

            builder.HasOne(x => x.Airport)
                .WithMany(x => x.Gates)
                .HasForeignKey(x => x.AirportId)
                .IsRequired();
        }
    }
}
