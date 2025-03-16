using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddAirwayAndWaypoints : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
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
                    section_code = table.Column<string>(type: "text", nullable: false),
                    airport_icao_ident = table.Column<string>(type: "text", nullable: true),
                    icao_code = table.Column<string>(type: "text", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_ndb_navaid", x => x.id);
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
                name: "vhf_navaid",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    icao_code = table.Column<string>(type: "text", nullable: false),
                    vor_identifier = table.Column<string>(type: "text", nullable: false),
                    vor_latitude = table.Column<double>(type: "double precision", nullable: false),
                    vor_longitude = table.Column<double>(type: "double precision", nullable: false),
                    dme_identifier = table.Column<string>(type: "text", nullable: true),
                    dme_latitude = table.Column<double>(type: "double precision", nullable: true),
                    dme_longitude = table.Column<double>(type: "double precision", nullable: true)
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
                    section_code = table.Column<string>(type: "text", nullable: false),
                    region_code = table.Column<string>(type: "text", nullable: false),
                    icao_code = table.Column<string>(type: "text", nullable: false),
                    identifier = table.Column<string>(type: "text", nullable: false),
                    latitude = table.Column<double>(type: "double precision", nullable: false),
                    longitude = table.Column<double>(type: "double precision", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_waypoint", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "airway_fix",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    airway_id = table.Column<Guid>(type: "uuid", nullable: false),
                    sequence_number = table.Column<long>(type: "bigint", nullable: false),
                    fix_identifier = table.Column<string>(type: "text", nullable: false),
                    fix_icao_code = table.Column<string>(type: "text", nullable: false),
                    description_code = table.Column<string>(type: "text", nullable: false)
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
                name: "ix_airway_fix_airway_id",
                schema: "navdata",
                table: "airway_fix",
                column: "airway_id");

            migrationBuilder.CreateIndex(
                name: "ix_procedure_airport_id",
                schema: "navdata",
                table: "procedure",
                column: "airport_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
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
                name: "vhf_navaid",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "waypoint",
                schema: "navdata");

            migrationBuilder.DropTable(
                name: "airway",
                schema: "navdata");
        }
    }
}
