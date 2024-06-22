using Microsoft.EntityFrameworkCore.Metadata.Builders;

namespace Net.Vatprc.Uniapi.Models;

public class Event
{
  public Ulid Id { get; set; } = Ulid.NewUlid();

  public string Title { get; set; } = string.Empty;

  public DateTimeOffset StartAt { get; set; }

  public DateTimeOffset EndAt { get; set; }

  public DateTimeOffset CreatedAt { get; set; }

  public DateTimeOffset UpdatedAt { get; set; }

  public class EventConfiguration : IEntityTypeConfiguration<Event>
  {
    public void Configure(EntityTypeBuilder<Event> builder)
    {
      builder.Property(x => x.CreatedAt)
          .HasDefaultValueSql("CURRENT_TIMESTAMP");

      builder.Property(x => x.UpdatedAt)
          .HasDefaultValueSql("CURRENT_TIMESTAMP")
          .ValueGeneratedOnAddOrUpdate();
    }
  }
}
