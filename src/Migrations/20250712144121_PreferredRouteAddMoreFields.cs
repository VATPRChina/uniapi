using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class PreferredRouteAddMoreFields : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<int[]>(
                name: "allowed_altitudes",
                schema: "navdata",
                table: "preferred_route",
                type: "integer[]",
                nullable: false,
                defaultValue: new int[0]);

            migrationBuilder.AddColumn<string>(
                name: "cruising_level_restriction",
                schema: "navdata",
                table: "preferred_route",
                type: "text",
                nullable: false,
                defaultValue: "");

            migrationBuilder.AddColumn<int>(
                name: "minimal_altitude",
                schema: "navdata",
                table: "preferred_route",
                type: "integer",
                nullable: false,
                defaultValue: 0);

            migrationBuilder.AddColumn<string>(
                name: "remarks",
                schema: "navdata",
                table: "preferred_route",
                type: "text",
                nullable: false,
                defaultValue: "");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "allowed_altitudes",
                schema: "navdata",
                table: "preferred_route");

            migrationBuilder.DropColumn(
                name: "cruising_level_restriction",
                schema: "navdata",
                table: "preferred_route");

            migrationBuilder.DropColumn(
                name: "minimal_altitude",
                schema: "navdata",
                table: "preferred_route");

            migrationBuilder.DropColumn(
                name: "remarks",
                schema: "navdata",
                table: "preferred_route");
        }
    }
}
