using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi;

public class Database(DbContextOptions<Database> options) : DbContext(options)
{
    public virtual DbSet<User> User { get; set; } = null!;
    public virtual DbSet<RefreshToken> Session { get; set; } = null!;

    public virtual DbSet<Event> Event { get; set; } = null!;
    public virtual DbSet<EventSlot> EventSlot { get; set; } = null!;
    public virtual DbSet<EventAirspace> EventAirspace { get; set; } = null!;
    public virtual DbSet<EventBooking> EventBooking { get; set; } = null!;
    public virtual DbSet<DeviceAuthorization> DeviceAuthorization { get; set; } = null!;

    public virtual DbSet<Flight> Flight { get; set; } = null!;

    public virtual DbSet<Airport> Airport { get; set; } = null!;
    public virtual DbSet<AirportGate> AirportGate { get; set; } = null!;
    public virtual DbSet<Airway> Airway { get; set; } = null!;
    public virtual DbSet<AirwayFix> AirwayFix { get; set; } = null!;
    public virtual DbSet<NdbNavaid> NdbNavaid { get; set; } = null!;
    public virtual DbSet<PreferredRoute> PreferredRoute { get; set; } = null!;
    public virtual DbSet<Procedure> Procedure { get; set; } = null!;
    public virtual DbSet<Runway> Runway { get; set; } = null!;
    public virtual DbSet<VhfNavaid> VhfNavaid { get; set; } = null!;
    public virtual DbSet<Waypoint> Waypoint { get; set; } = null!;

    public virtual DbSet<UserAtcPermission> UserAtcPermission { get; set; } = null!;

    public virtual DbSet<Sheet> Sheet { get; set; } = null!;
    public virtual DbSet<SheetField> SheetField { get; set; } = null!;
    public virtual DbSet<SheetFiling> SheetFiling { get; set; } = null!;
    public virtual DbSet<SheetFilingAnswer> SheetFilingAnswer { get; set; } = null!;

    public Database() : this(new DbContextOptions<Database>())
    {
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
