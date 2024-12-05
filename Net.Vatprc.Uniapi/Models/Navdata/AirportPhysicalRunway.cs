using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models.Navdata;

public class AirportPhysicalRunway
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public Ulid AirportId { get; set; }
    public Airport Airport { get; set; } = null!;

    public Ulid Runway1Id { get; set; }
    public Runway? Runway1 { get; set; }

    public Ulid Runway2Id { get; set; }
    public Runway? Runway2 { get; set; }

    public class Configuration : IEntityTypeConfiguration<AirportPhysicalRunway>
    {
        public void Configure(EntityTypeBuilder<AirportPhysicalRunway> builder)
        {
            builder.ToTable("airport_physical_runway", "navdata");

            builder.Property(x => x.Runway1Id).HasColumnName("runway1_id");
            builder.Property(x => x.Runway2Id).HasColumnName("runway2_id");

            builder.HasOne(x => x.Airport)
                .WithMany(x => x.PhysicalRunways)
                .HasForeignKey(x => x.AirportId)
                .IsRequired();

            builder.HasOne(x => x.Runway1)
                .WithOne()
                .HasForeignKey<AirportPhysicalRunway>(x => x.Runway1Id)
                .IsRequired();

            builder.HasOne(x => x.Runway2)
                .WithOne()
                .HasForeignKey<AirportPhysicalRunway>(x => x.Runway2Id)
                .IsRequired();
        }
    }
}
