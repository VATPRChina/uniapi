using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class EventAddLinks : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "community_link",
                table: "event",
                type: "text",
                nullable: true);

            migrationBuilder.AddColumn<string>(
                name: "vatsim_link",
                table: "event",
                type: "text",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "community_link",
                table: "event");

            migrationBuilder.DropColumn(
                name: "vatsim_link",
                table: "event");
        }
    }
}
