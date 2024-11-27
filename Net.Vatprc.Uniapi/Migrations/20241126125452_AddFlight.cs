using System;
using Microsoft.EntityFrameworkCore.Migrations;
using Net.Vatprc.Uniapi.Models.Acdm;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddFlight : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterDatabase()
                .Annotation("Npgsql:Enum:flight_state", "approach,climb,cruise,descent,landing,pre_departure,pushback,shutdown,takeoff,taxi,taxi_arrival");

            migrationBuilder.CreateTable(
                name: "flight",
                columns: table => new
                {
                    callsign = table.Column<string>(type: "text", nullable: false),
                    cid = table.Column<string>(type: "text", nullable: false),
                    last_observed_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false),
                    altitude = table.Column<long>(type: "bigint", nullable: false),
                    departure = table.Column<string>(type: "text", nullable: false),
                    arrival = table.Column<string>(type: "text", nullable: false),
                    cruise_tas = table.Column<long>(type: "bigint", nullable: false),
                    raw_route = table.Column<string>(type: "text", nullable: false),
                    state = table.Column<Flight.FlightState>(type: "flight_state", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_flight", x => x.callsign);
                });

            migrationBuilder.CreateIndex(
                name: "ix_flight_cid",
                table: "flight",
                column: "cid",
                unique: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "flight");

            migrationBuilder.AlterDatabase()
                .OldAnnotation("Npgsql:Enum:flight_state", "approach,climb,cruise,descent,landing,pre_departure,pushback,shutdown,takeoff,taxi,taxi_arrival");
        }
    }
}
