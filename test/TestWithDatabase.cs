using Microsoft.EntityFrameworkCore;

namespace Net.Vatprc.Uniapi.Test;

public class TestWithDatabase
{
    protected Database dbContext;

    [SetUp]
    protected void DatabaseSetup()
    {
        dbContext = new Database(new DbContextOptionsBuilder<Database>()
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
