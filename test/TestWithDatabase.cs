using Microsoft.EntityFrameworkCore;
using Net.Vatprc.Uniapi.Adapters;

namespace Net.Vatprc.Uniapi.Test;

public abstract class TestWithDatabase
{
    protected DatabaseAdapter dbContext;

    [SetUp]
    protected void DatabaseSetup()
    {
        dbContext = new DatabaseAdapter(new DbContextOptionsBuilder<DatabaseAdapter>()
            .UseSnakeCaseNamingConvention()
            .UseNpgsql($"Host=localhost;Username=postgres;Database=vatprc-test-{Ulid.NewUlid()}")
            .Options);

        dbContext.Database.EnsureCreated();
    }

    [TearDown]
    protected void DatabaseTeardown()
    {
        dbContext.Database.EnsureDeleted();
        dbContext.Dispose();
    }
}
