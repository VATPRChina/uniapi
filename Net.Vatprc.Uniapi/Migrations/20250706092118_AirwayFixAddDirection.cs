using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AirwayFixAddDirection : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<char>(
                name: "directional_restriction",
                schema: "navdata",
                table: "airway_fix",
                type: "character(1)",
                nullable: false,
                defaultValue: ' ');
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "directional_restriction",
                schema: "navdata",
                table: "airway_fix");
        }
    }
}
