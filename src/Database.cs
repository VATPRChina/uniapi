using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Event;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi;

public class Database(DbContextOptions<Database> options) : DbContext(options)
{
    public virtual DbSet<User> User { get; set; }
    public virtual DbSet<RefreshToken> Session { get; set; }
    public virtual DbSet<DeviceAuthorization> DeviceAuthorization { get; set; }

    public virtual DbSet<Event> Event { get; set; }
    public virtual DbSet<EventSlot> EventSlot { get; set; }
    public virtual DbSet<EventAirspace> EventAirspace { get; set; }
    public virtual DbSet<EventBooking> EventBooking { get; set; }
    public virtual DbSet<EventAtcPosition> EventAtcPosition { get; set; }

    public virtual DbSet<Flight> Flight { get; set; }

    public virtual DbSet<Airport> Airport { get; set; }
    public virtual DbSet<AirportGate> AirportGate { get; set; }
    public virtual DbSet<Airway> Airway { get; set; }
    public virtual DbSet<AirwayFix> AirwayFix { get; set; }
    public virtual DbSet<NdbNavaid> NdbNavaid { get; set; }
    public virtual DbSet<PreferredRoute> PreferredRoute { get; set; }
    public virtual DbSet<Procedure> Procedure { get; set; }
    public virtual DbSet<Runway> Runway { get; set; }
    public virtual DbSet<VhfNavaid> VhfNavaid { get; set; }
    public virtual DbSet<Waypoint> Waypoint { get; set; }

    public virtual DbSet<UserAtcPermission> UserAtcPermission { get; set; }
    public virtual DbSet<AtcApplication> AtcApplication { get; set; }

    public virtual DbSet<Sheet> Sheet { get; set; }
    public virtual DbSet<SheetField> SheetField { get; set; }
    public virtual DbSet<SheetFiling> SheetFiling { get; set; }
    public virtual DbSet<SheetFilingAnswer> SheetFilingAnswer { get; set; }

    protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
    {
        optionsBuilder.UseSnakeCaseNamingConvention();
    }

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
