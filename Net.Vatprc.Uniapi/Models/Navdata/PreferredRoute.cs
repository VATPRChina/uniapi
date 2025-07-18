using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// This class do not adhere to ARINC 424 specification.
/// </summary>
public class PreferredRoute
{
    public Ulid Id { get; set; } = Ulid.NewUlid();
    public string Departure { get; set; } = string.Empty;
    public string Arrival { get; set; } = string.Empty;
    public string RawRoute { get; set; } = string.Empty;
    public LevelRestrictionType CruisingLevelRestriction { get; set; } = LevelRestrictionType.Standard;
    public IEnumerable<int> AllowedAltitudes { get; set; } = [];
    public int MinimalAltitude { get; set; } = 0;
    public string Remarks { get; set; } = string.Empty;

    public enum LevelRestrictionType
    {
        StandardEven,
        StandardOdd,
        Standard,
        FlightLevelEven,
        FlightLevelOdd,
        FlightLevel,
    }

    public class Configuration : IEntityTypeConfiguration<PreferredRoute>
    {
        public void Configure(EntityTypeBuilder<PreferredRoute> builder)
        {
            builder.ToTable("preferred_route", "navdata");

            builder.Property(p => p.CruisingLevelRestriction)
                .HasConversion<string>();
        }
    }
}
