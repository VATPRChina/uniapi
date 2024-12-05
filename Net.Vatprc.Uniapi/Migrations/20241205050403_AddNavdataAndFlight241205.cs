using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddNavdataAndFlight241205 : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.EnsureSchema(
                name: "navdata");

            migrationBuilder.CreateTable(
                name: "airport",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false),
                    elevation = table.Column<int>(type: "integer", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_airport", x => x.id);
                });

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
                    departure_gate = table.Column<string>(type: "text", nullable: true),
                    arrival = table.Column<string>(type: "text", nullable: false),
                    arrival_gate = table.Column<string>(type: "text", nullable: true),
                    cruise_tas = table.Column<long>(type: "bigint", nullable: false),
                    raw_route = table.Column<string>(type: "text", nullable: false),
                    state = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_flight", x => x.callsign);
                });

            migrationBuilder.CreateTable(
                name: "airport_gate",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    airport_id = table.Column<Guid>(type: "uuid", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_airport_gate", x => x.id);
                    table.ForeignKey(
                        name: "fk_airport_gate_airport_airport_id",
                        column: x => x.airport_id,
                        principalSchema: "navdata",
                        principalTable: "airport",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "runway",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    airport_id = table.Column<Guid>(type: "uuid", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_runway", x => x.id);
                    table.ForeignKey(
                        name: "fk_runway_airport_airport_id",
                        column: x => x.airport_id,
                        principalSchema: "navdata",
                        principalTable: "airport",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "airport_physical_runway",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    airport_id = table.Column<Guid>(type: "uuid", nullable: false),
                    runway1_id = table.Column<Guid>(type: "uuid", nullable: false),
                    runway2_id = table.Column<Guid>(type: "uuid", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_airport_physical_runway", x => x.id);
                    table.ForeignKey(
                        name: "fk_airport_physical_runway_airport_airport_id",
                        column: x => x.airport_id,
                        principalSchema: "navdata",
                        principalTable: "airport",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_airport_physical_runway_runway_runway1id",
                        column: x => x.runway1_id,
                        principalSchema: "navdata",
                        principalTable: "runway",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_airport_physical_runway_runway_runway2id",
                        column: x => x.runway2_id,
                        principalSchema: "navdata",
                        principalTable: "runway",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_airport_identifier",
                schema: "navdata",
                table: "airport",
                column: "identifier",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "ix_airport_gate_airport_id",
                schema: "navdata",
                table: "airport_gate",
                column: "airport_id");

            migrationBuilder.CreateIndex(
                name: "ix_airport_physical_runway_airport_id",
                schema: "navdata",
                table: "airport_physical_runway",
                column: "airport_id");

            migrationBuilder.CreateIndex(
                name: "ix_airport_physical_runway_runway1id",
                schema: "navdata",
                table: "airport_physical_runway",
                column: "runway1_id",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "ix_airport_physical_runway_runway2id",
                schema: "navdata",
                table: "airport_physical_runway",
                column: "runway2_id",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "ix_flight_cid",
                table: "flight",
                column: "cid",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "ix_runway_airport_id",
                schema: "navdata",
                table: "runway",
                column: "airport_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "airport_gate",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "airport_physical_runway",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "flight");

            migrationBuilder.DropTable(
                name: "runway",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "airport",
                schema: "navdata");
        }
    }
}
