using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class SessionAddCode : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<Guid>(
                name: "code",
                table: "session",
                type: "uuid",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "code",
                table: "session");
        }
    }
}
