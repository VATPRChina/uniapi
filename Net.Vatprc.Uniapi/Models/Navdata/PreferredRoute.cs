using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

public class PreferredRoute
{
    public Ulid Id { get; set; } = Ulid.NewUlid();
    public string Departure { get; set; } = string.Empty;
    public string Arrival { get; set; } = string.Empty;
    public string RawRoute { get; set; } = string.Empty;

    public class Configuration : IEntityTypeConfiguration<PreferredRoute>
    {
        public void Configure(EntityTypeBuilder<PreferredRoute> builder)
        {
            builder.ToTable("preferred_route", "navdata");
        }
    }
}
