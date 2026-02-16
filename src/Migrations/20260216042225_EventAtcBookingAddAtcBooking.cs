using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class EventAtcBookingAddAtcBooking : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<Guid>(
                name: "atc_booking_id",
                table: "event_atc_position_booking",
                type: "uuid",
                nullable: true);

            migrationBuilder.CreateIndex(
                name: "ix_event_atc_position_booking_atc_booking_id",
                table: "event_atc_position_booking",
                column: "atc_booking_id",
                unique: true);

            migrationBuilder.AddForeignKey(
                name: "fk_event_atc_position_booking_atc_booking_atc_booking_id",
                table: "event_atc_position_booking",
                column: "atc_booking_id",
                principalTable: "atc_booking",
                principalColumn: "id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_event_atc_position_booking_atc_booking_atc_booking_id",
                table: "event_atc_position_booking");

            migrationBuilder.DropIndex(
                name: "ix_event_atc_position_booking_atc_booking_id",
                table: "event_atc_position_booking");

            migrationBuilder.DropColumn(
                name: "atc_booking_id",
                table: "event_atc_position_booking");
        }
    }
}
