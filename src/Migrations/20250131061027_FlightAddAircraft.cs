using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class FlightAddAircraft : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "aircraft",
                table: "flight",
                type: "text",
                nullable: false,
                defaultValue: "");

            migrationBuilder.AddColumn<string>(
                name: "equipment",
                table: "flight",
                type: "text",
                nullable: false,
                defaultValue: "");

            migrationBuilder.AddColumn<string>(
                name: "navigation_performance",
                table: "flight",
                type: "text",
                nullable: false,
                defaultValue: "");

            migrationBuilder.AddColumn<string>(
                name: "transponder",
                table: "flight",
                type: "text",
                nullable: false,
                defaultValue: "");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "aircraft",
                table: "flight");

            migrationBuilder.DropColumn(
                name: "equipment",
                table: "flight");

            migrationBuilder.DropColumn(
                name: "navigation_performance",
                table: "flight");

            migrationBuilder.DropColumn(
                name: "transponder",
                table: "flight");
        }
    }
}
