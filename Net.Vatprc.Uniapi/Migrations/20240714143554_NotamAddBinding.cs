using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class NotamAddBinding : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "notam_binding",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    notam_id = table.Column<Guid>(type: "uuid", nullable: false),
                    created_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false, defaultValueSql: "CURRENT_TIMESTAMP"),
                    updated_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false, defaultValueSql: "CURRENT_TIMESTAMP"),
                    discriminator = table.Column<string>(type: "character varying(34)", maxLength: 34, nullable: false),
                    event_id = table.Column<Guid>(type: "uuid", nullable: true),
                    event_airspace_id = table.Column<Guid>(type: "uuid", nullable: true),
                    icao_code = table.Column<string>(type: "text", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_notam_binding", x => x.id);
                    table.ForeignKey(
                        name: "fk_notam_binding_event_airspace_event_airspace_id",
                        column: x => x.event_airspace_id,
                        principalTable: "event_airspace",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_notam_binding_event_event_id",
                        column: x => x.event_id,
                        principalTable: "event",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_notam_binding_notam_notam_id",
                        column: x => x.notam_id,
                        principalTable: "notam",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_notam_binding_event_airspace_id",
                table: "notam_binding",
                column: "event_airspace_id");

            migrationBuilder.CreateIndex(
                name: "ix_notam_binding_event_id",
                table: "notam_binding",
                column: "event_id");

            migrationBuilder.CreateIndex(
                name: "ix_notam_binding_icao_code",
                table: "notam_binding",
                column: "icao_code");

            migrationBuilder.CreateIndex(
                name: "ix_notam_binding_notam_id",
                table: "notam_binding",
                column: "notam_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "notam_binding");
        }
    }
}
