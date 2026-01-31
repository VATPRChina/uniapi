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
    public virtual DbSet<EventAtcPositionBooking> EventAtcPositionBooking { get; set; }

    public virtual DbSet<PreferredRoute> PreferredRoute { get; set; }

    public virtual DbSet<UserAtcPermission> UserAtcPermission { get; set; }
    public virtual DbSet<UserAtcStatus> UserAtcStatus { get; set; }
    public virtual DbSet<AtcApplication> AtcApplication { get; set; }
    public virtual DbSet<AtcBooking> AtcBooking { get; set; }
    public virtual DbSet<Training> Training { get; set; }
    public virtual DbSet<TrainingApplication> TrainingApplication { get; set; }
    public virtual DbSet<TrainingApplicationSlot> TrainingApplicationSlot { get; set; }
    public virtual DbSet<TrainingApplicationResponse> TrainingApplicationResponse { get; set; }


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
