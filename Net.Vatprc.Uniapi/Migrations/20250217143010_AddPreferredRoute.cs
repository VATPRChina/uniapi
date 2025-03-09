using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddPreferredRoute : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "preferred_route",
                schema: "navdata",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    departure = table.Column<string>(type: "text", nullable: false),
                    arrival = table.Column<string>(type: "text", nullable: false),
                    raw_route = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_preferred_route", x => x.id);
                });
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "preferred_route",
                schema: "navdata");
        }
    }
}
