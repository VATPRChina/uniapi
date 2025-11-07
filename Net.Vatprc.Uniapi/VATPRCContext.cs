using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi;

public class VATPRCContext(DbContextOptions<VATPRCContext> options) : DbContext(options)
{
    public DbSet<User> User { get; set; } = null!;
    public DbSet<RefreshToken> Session { get; set; } = null!;

    public DbSet<Event> Event { get; set; } = null!;
    public DbSet<EventSlot> EventSlot { get; set; } = null!;
    public DbSet<EventAirspace> EventAirspace { get; set; } = null!;
    public DbSet<EventBooking> EventBooking { get; set; } = null!;
    public DbSet<DeviceAuthorization> DeviceAuthorization { get; set; } = null!;

    public DbSet<Flight> Flight { get; set; } = null!;

    public DbSet<Airport> Airport { get; set; } = null!;
    public DbSet<AirportGate> AirportGate { get; set; } = null!;
    public DbSet<Airway> Airway { get; set; } = null!;
    public DbSet<AirwayFix> AirwayFix { get; set; } = null!;
    public DbSet<NdbNavaid> NdbNavaid { get; set; } = null!;
    public DbSet<PreferredRoute> PreferredRoute { get; set; } = null!;
    public DbSet<Procedure> Procedure { get; set; } = null!;
    public DbSet<Runway> Runway { get; set; } = null!;
    public DbSet<VhfNavaid> VhfNavaid { get; set; } = null!;
    public DbSet<Waypoint> Waypoint { get; set; } = null!;

    public DbSet<UserAtcPermission> UserAtcPermission { get; set; } = null!;

    public DbSet<Sheet> Sheet { get; set; } = null!;
    public DbSet<SheetField> SheetField { get; set; } = null!;
    public DbSet<SheetFiling> SheetFiling { get; set; } = null!;
    public DbSet<SheetFilingAnswer> SheetFilingAnswer { get; set; } = null!;

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
