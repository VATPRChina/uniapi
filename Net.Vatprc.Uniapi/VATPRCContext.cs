using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi;

public class VATPRCContext(DbContextOptions<VATPRCContext> options) : DbContext(options)
{
    public DbSet<User> User { get; set; } = null!;
    public DbSet<Session> Session { get; set; } = null!;
    public DbSet<Event> Event { get; set; } = null!;
    public DbSet<EventSlot> EventSlot { get; set; } = null!;
    public DbSet<EventAirspace> EventAirspace { get; set; } = null!;
    public DbSet<EventBooking> EventBooking { get; set; } = null!;
    public DbSet<Notam> Notam { get; set; } = null!;
    public DbSet<DeviceAuthorization> DeviceAuthorization { get; set; } = null!;

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        modelBuilder.ApplyConfigurationsFromAssembly(
            System.Reflection.Assembly.GetExecutingAssembly());
    }

    protected override void ConfigureConventions(ModelConfigurationBuilder configurationBuilder)
    {
        configurationBuilder
            .Properties<Ulid>()
            .HaveConversion<UlidToGuidConverter>();
    }
}
