using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class EventSlotAddCallsignAndAircraftType : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "aircraft_type_icao",
                table: "event_slot",
                type: "text",
                nullable: true);

            migrationBuilder.AddColumn<string>(
                name: "callsign",
                table: "event_slot",
                type: "text",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "aircraft_type_icao",
                table: "event_slot");

            migrationBuilder.DropColumn(
                name: "callsign",
                table: "event_slot");
        }
    }
}
