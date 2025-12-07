using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class PreferredRouteAddTimeFrame : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<DateTimeOffset>(
                name: "valid_from",
                schema: "navdata",
                table: "preferred_route",
                type: "timestamp with time zone",
                nullable: true);

            migrationBuilder.AddColumn<DateTimeOffset>(
                name: "valid_until",
                schema: "navdata",
                table: "preferred_route",
                type: "timestamp with time zone",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "valid_from",
                schema: "navdata",
                table: "preferred_route");

            migrationBuilder.DropColumn(
                name: "valid_until",
                schema: "navdata",
                table: "preferred_route");
        }
    }
}
