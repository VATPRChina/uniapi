using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class FlightAddIdAndFinalizedAt : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropPrimaryKey(
                name: "pk_flight",
                table: "flight");

            migrationBuilder.DropIndex(
                name: "ix_flight_cid",
                table: "flight");

            migrationBuilder.AddColumn<Guid>(
                name: "id",
                table: "flight",
                type: "uuid",
                nullable: false,
                defaultValue: new Guid("00000000-0000-0000-0000-000000000000"));

            migrationBuilder.AddColumn<string>(
                name: "arrival_runway",
                table: "flight",
                type: "text",
                nullable: true);

            migrationBuilder.AddColumn<string>(
                name: "departure_runway",
                table: "flight",
                type: "text",
                nullable: true);

            migrationBuilder.AddColumn<DateTimeOffset>(
                name: "finalized_at",
                table: "flight",
                type: "timestamp with time zone",
                nullable: true);

            migrationBuilder.AddPrimaryKey(
                name: "pk_flight",
                table: "flight",
                column: "id");

            migrationBuilder.CreateIndex(
                name: "ix_flight_callsign",
                table: "flight",
                column: "callsign");

            migrationBuilder.CreateIndex(
                name: "ix_flight_cid",
                table: "flight",
                column: "cid");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropPrimaryKey(
                name: "pk_flight",
                table: "flight");

            migrationBuilder.DropIndex(
                name: "ix_flight_callsign",
                table: "flight");

            migrationBuilder.DropIndex(
                name: "ix_flight_cid",
                table: "flight");

            migrationBuilder.DropColumn(
                name: "id",
                table: "flight");

            migrationBuilder.DropColumn(
                name: "arrival_runway",
                table: "flight");

            migrationBuilder.DropColumn(
                name: "departure_runway",
                table: "flight");

            migrationBuilder.DropColumn(
                name: "finalized_at",
                table: "flight");

            migrationBuilder.AddPrimaryKey(
                name: "pk_flight",
                table: "flight",
                column: "callsign");

            migrationBuilder.CreateIndex(
                name: "ix_flight_cid",
                table: "flight",
                column: "cid",
                unique: true);
        }
    }
}
