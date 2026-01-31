using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// ARINC 424 waypoint primary record. (Section 4.1.4.1)
/// </summary>
public class Waypoint
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    /// <summary>
    /// Section and subsection code. (Section 5.4, 5.5)
    /// </summary>
    public string SectionCode { get; set; } = string.Empty;

    public bool IsEnroute => SectionCode == "EA";
    public bool IsTerminal => SectionCode == "PC";

    /// <summary>
    /// "ENRT" for enroute waypoints, or airport ICAO identifier for terminal
    /// waypoints. (Section 5.41)
    /// </summary>
    public string RegionCode { get; set; } = string.Empty;

    /// <summary>
    /// ICAO code. (Section 5.33)
    /// </summary>
    public string IcaoCode { get; set; } = string.Empty;

    /// <summary>
    /// Waypoint identifier. (Section 5.13)
    /// </summary>
    public string Identifier { get; set; } = string.Empty;

    /// <summary>
    /// Waypoint latitude. (Section 5.36)
    /// </summary>
    public double Latitude { get; set; }

    /// <summary>
    /// Waypoint longitude. (Section 5.37)
    /// </summary>
    public double Longitude { get; set; }

    public class Configuration : IEntityTypeConfiguration<Waypoint>
    {
        public void Configure(EntityTypeBuilder<Waypoint> builder)
        {
            builder.ToTable("waypoint", "navdata");
        }
    }
}
