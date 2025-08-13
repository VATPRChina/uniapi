using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class RemoveAirportPhysicalRunway : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "airport_physical_runway",
                schema: "navdata");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
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
        }
    }
}
