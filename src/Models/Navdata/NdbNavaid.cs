using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// ARINC 424 NDB navaid primary record. (Section 4.1.3.1)
/// </summary>
public class NdbNavaid
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    /// <summary>
    /// Section and subsection code. (Section 5.4, 5.5)
    /// </summary>
    public string SectionCode { get; set; } = string.Empty;

    public bool IsEnroute => SectionCode == "DB";
    public bool IsTerminal => SectionCode == "PN";

    /// <summary>
    /// Airport ICAO identifier. (Section 5.6)
    /// </summary>
    public string? AirportIcaoIdent { get; set; }

    /// <summary>
    /// ICAO code. (Section 5.33)
    /// </summary>
    public string IcaoCode { get; set; } = string.Empty;

    /// <summary>
    /// NDB identifier. (Section 5.33)
    /// </summary>
    public string Identifier { get; set; } = string.Empty;

    /// <summary>
    /// NDB latitude. (Section 5.36)
    /// </summary>
    public double Latitude { get; set; }

    /// <summary>
    /// NDB longitude. (Section 5.37)
    /// </summary>
    public double Longitude { get; set; }

    public class Configuration : IEntityTypeConfiguration<NdbNavaid>
    {
        public void Configure(EntityTypeBuilder<NdbNavaid> builder)
        {
            builder.ToTable("ndb_navaid", "navdata");
        }
    }
}
