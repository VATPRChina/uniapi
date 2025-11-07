using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// ARINC 424 Enroute airways primary record. (Section 4.1.6.1)
/// </summary>
public class AirwayFix
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    /// <summary>
    /// Airway Identifier. (Section 5.8)
    /// </summary>
    public string? AirwayIdentifier => Airway?.Identifier;

    public Ulid AirwayId { get; set; }
    public Airway? Airway { get; set; }

    /// <summary>
    /// Sequence number. (Section 5.12)
    /// </summary>
    public uint SequenceNumber { get; set; }

    /// <summary>
    /// Fix identifier. (Section 5.13)
    /// </summary>
    public string FixIdentifier { get; set; } = string.Empty;

    /// <summary>
    /// Fix ICAO code. (Section 5.14)
    /// </summary>
    public string FixIcaoCode { get; set; } = string.Empty;

    /// <summary>
    /// Waypoint description code. (Section 5.17)
    /// </summary>
    public string DescriptionCode { get; set; } = string.Empty;

    /// <summary>
    /// Directional Restriction. (Section 5.115)
    /// </summary>
    public char DirectionalRestriction { get; set; } = ' ';

    public class Configuration : IEntityTypeConfiguration<AirwayFix>
    {
        public void Configure(EntityTypeBuilder<AirwayFix> builder)
        {
            builder.ToTable("airway_fix", "navdata");

            builder.HasOne(x => x.Airway)
                .WithMany(x => x.Fixes)
                .HasForeignKey(x => x.AirwayId)
                .IsRequired();

            builder.Property(x => x.DirectionalRestriction)
                .HasDefaultValue(' ');
        }
    }
}
