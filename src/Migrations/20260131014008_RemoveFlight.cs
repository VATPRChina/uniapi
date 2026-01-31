using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class RemoveFlight : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "flight");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "flight",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    aircraft = table.Column<string>(type: "text", nullable: false),
                    altitude = table.Column<long>(type: "bigint", nullable: false),
                    arrival = table.Column<string>(type: "text", nullable: false),
                    arrival_gate = table.Column<string>(type: "text", nullable: true),
                    arrival_runway = table.Column<string>(type: "text", nullable: true),
                    callsign = table.Column<string>(type: "text", nullable: false),
                    cid = table.Column<string>(type: "text", nullable: false),
                    cruise_tas = table.Column<long>(type: "bigint", nullable: false),
                    cruising_level = table.Column<long>(type: "bigint", nullable: false),
                    departure = table.Column<string>(type: "text", nullable: false),
                    departure_gate = table.Column<string>(type: "text", nullable: true),
                    departure_runway = table.Column<string>(type: "text", nullable: true),
                    equipment = table.Column<string>(type: "text", nullable: false),
                    finalized_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: true),
                    last_observed_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false),
                    navigation_performance = table.Column<string>(type: "text", nullable: false),
                    raw_route = table.Column<string>(type: "text", nullable: false),
                    state = table.Column<string>(type: "text", nullable: false),
                    transponder = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_flight", x => x.id);
                });

            migrationBuilder.CreateIndex(
                name: "ix_flight_callsign",
                table: "flight",
                column: "callsign");

            migrationBuilder.CreateIndex(
                name: "ix_flight_cid",
                table: "flight",
                column: "cid");
        }
    }
}
