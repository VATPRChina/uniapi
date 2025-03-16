using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// ARINC424 VHF navaid primary record. (Section 4.1.2.1)
/// </summary>
public class VhfNavaid
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    /// <summary>
    /// (Section 5.14)
    /// 
    /// Definition/Description: The ICAO Code field permits records to be categorized
    /// geographically within the limits of the categorization performed by the Area
    /// Code field.
    ///
    /// Source/Content: The code is to be employed in the ICAO code field may be
    /// found in ICAO Document No. 7910, Location Indicators.
    /// </summary>
    public string IcaoCode { get; set; } = string.Empty;

    /// <summary>
    /// The VOR/NDB Identifier field identifies the VHF/MF/LF facility defined in
    /// the record. (Section 5.33)
    /// </summary>
    public string VorIdentifier { get; set; } = string.Empty;

    /// <summary>
    /// VOR's latitude. (Section 5.36)
    /// </summary>
    public double VorLatitude { get; set; }

    /// <summary>
    /// VOR's longitude. (Section 5.37)
    /// </summary>
    public double VorLongitude { get; set; }

    /// <summary>
    /// The identification of a DME facility, a TACAN facility or the DME (or TACAN)
    /// component of a VORDME or VORTAC facility. (Section 5.38)
    /// </summary>
    public string? DmeIdentifier { get; set; }

    /// <summary>
    /// DME's latitude. (Section 5.36)
    /// </summary>
    public double? DmeLatitude { get; set; }

    /// <summary>
    /// DME's longitude. (Section 5.37)
    /// </summary>
    public double? DmeLongitude { get; set; }

    public class Configuration : IEntityTypeConfiguration<VhfNavaid>
    {
        public void Configure(EntityTypeBuilder<VhfNavaid> builder)
        {
            builder.ToTable("vhf_navaid", "navdata");
        }
    }
}
