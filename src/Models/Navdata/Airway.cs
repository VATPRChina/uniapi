using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

/// <summary>
/// A continous segment of ARINC 424 enroute airway.
/// </summary>
public class Airway
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    /// <summary>
    /// Route identifier. (Section 5.8)
    /// </summary>
    public string Identifier { get; set; } = string.Empty;

    public IList<AirwayFix> Fixes { get; set; } = [];

    public class Configuration : IEntityTypeConfiguration<Airway>
    {
        public void Configure(EntityTypeBuilder<Airway> builder)
        {
            builder.ToTable("airway", "navdata");
        }
    }
}
