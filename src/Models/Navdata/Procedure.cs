using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// Airport SID/STAR/Approach Primary Records in ARINC 424 Section 4.1.9.1.
/// </summary>
public class Procedure
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
    /// SID/STAR Route Identifier (Section 5.9), or Approach Route Identifier (Section 5.10).
    /// </summary>
    public string Identifier { get; set; } = string.Empty;

    /// <summary>
    /// The Subsection Code field defines the specific part of the database major
    /// section in which the record resides.
    /// </summary>
    /// <remarks>
    /// `D` for SIDs. `E` for STARs. `F` for Approaches.
    /// </remarks>
    public char SubsectionCode { get; set; }

    public bool IsSid => SubsectionCode == 'D';
    public bool IsStar => SubsectionCode == 'E';
    public bool IsApproach => SubsectionCode == 'F';

    public class Configuration : IEntityTypeConfiguration<Procedure>
    {
        public void Configure(EntityTypeBuilder<Procedure> builder)
        {
            builder.ToTable("procedure", "navdata");

            builder.HasOne(x => x.Airport)
                .WithMany(x => x.Procedures)
                .HasForeignKey(x => x.AirportId)
                .IsRequired();
        }
    }
}
