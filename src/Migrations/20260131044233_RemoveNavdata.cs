using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class RemoveNavdata : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "airport_gate",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "airway_fix",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "ndb_navaid",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "procedure",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "runway",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "vhf_navaid",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "waypoint",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "airway",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "airport",
                schema: "navdata");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "airport",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    elevation = table.Column<int>(type: "integer", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_airport", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "airway",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_airway", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "ndb_navaid",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    airport_icao_ident = table.Column<string>(type: "text", nullable: true),
                    icao_code = table.Column<string>(type: "text", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false),
                    section_code = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_ndb_navaid", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "vhf_navaid",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    dme_identifier = table.Column<string>(type: "text", nullable: true),
                    dme_latitude = table.Column<double>(type: "double precision", nullable: true),
                    dme_longitude = table.Column<double>(type: "double precision", nullable: true),
                    icao_code = table.Column<string>(type: "text", nullable: false),
                    vor_identifier = table.Column<string>(type: "text", nullable: false),
                    vor_latitude = table.Column<double>(type: "double precision", nullable: true),
                    vor_longitude = table.Column<double>(type: "double precision", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_vhf_navaid", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "waypoint",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    icao_code = table.Column<string>(type: "text", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false),
                    region_code = table.Column<string>(type: "text", nullable: false),
                    section_code = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_waypoint", x => x.id);
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
                name: "procedure",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    airport_id = table.Column<Guid>(type: "uuid", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    subsection_code = table.Column<char>(type: "character(1)", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_procedure", x => x.id);
                    table.ForeignKey(
                        name: "fk_procedure_airport_airport_id",
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
                name: "airway_fix",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    airway_id = table.Column<Guid>(type: "uuid", nullable: false),
                    description_code = table.Column<string>(type: "text", nullable: false),
                    directional_restriction = table.Column<char>(type: "character(1)", nullable: false, defaultValue: ' '),
                    fix_icao_code = table.Column<string>(type: "text", nullable: false),
                    fix_identifier = table.Column<string>(type: "text", nullable: false),
                    sequence_number = table.Column<long>(type: "bigint", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_airway_fix", x => x.id);
                    table.ForeignKey(
                        name: "fk_airway_fix_airway_airway_id",
                        column: x => x.airway_id,
                        principalSchema: "navdata",
                        principalTable: "airway",
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
                name: "ix_airway_fix_airway_id",
                schema: "navdata",
                table: "airway_fix",
                column: "airway_id");

            migrationBuilder.CreateIndex(
                name: "ix_procedure_airport_id",
                schema: "navdata",
                table: "procedure",
                column: "airport_id");

            migrationBuilder.CreateIndex(
                name: "ix_runway_airport_id",
                schema: "navdata",
                table: "runway",
                column: "airport_id");
        }
    }
}
