using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public abstract class NotamBinding
{
    public Ulid Id { get; set; } = Ulid.NewUlid();

    public Ulid NotamId { get; set; }
    public Notam Notam { get; set; } = null!;

    public DateTimeOffset CreatedAt { get; set; }

    public DateTimeOffset UpdatedAt { get; set; }

    public class NotamBindingConfiguration : IEntityTypeConfiguration<NotamBinding>
    {
        public void Configure(EntityTypeBuilder<NotamBinding> builder)
        {
            builder.Property(x => x.CreatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP");

            builder.Property(x => x.UpdatedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP")
                .ValueGeneratedOnAddOrUpdate();

            builder.HasOne(x => x.Notam)
                .WithMany(x => x.Bindings)
                .HasForeignKey(x => x.NotamId);
        }
    }
}

public class NotamBindingIcaoCode : NotamBinding
{
    public string IcaoCode { get; set; } = string.Empty;

    public class NotamBindingIcaoCodeConfiguration : IEntityTypeConfiguration<NotamBindingIcaoCode>
    {
        public void Configure(EntityTypeBuilder<NotamBindingIcaoCode> builder)
        {
            builder.Property(x => x.IcaoCode).IsRequired();
            builder.HasIndex(x => x.IcaoCode);
        }
    }
}

public class NotamBindingEvent : NotamBinding
{
    public Ulid EventId { get; set; }
    public Event Event { get; set; } = null!;

    public class NotamBindingEventConfiguration : IEntityTypeConfiguration<NotamBindingEvent>
    {
        public void Configure(EntityTypeBuilder<NotamBindingEvent> builder)
        {
            builder.HasOne(x => x.Event)
                .WithMany()
                .HasForeignKey(x => x.EventId)
                .IsRequired();
        }
    }
}

public class NotamBindingEventAirspace : NotamBinding
{
    public Ulid EventAirspaceId { get; set; }
    public EventAirspace EventAirspace { get; set; } = null!;

    public class NotamBindingEventAirspaceConfiguration : IEntityTypeConfiguration<NotamBindingEventAirspace>
    {
        public void Configure(EntityTypeBuilder<NotamBindingEventAirspace> builder)
        {
            builder.HasOne(x => x.EventAirspace)
                .WithMany()
                .HasForeignKey(x => x.EventAirspaceId)
                .IsRequired();
        }
    }
}
