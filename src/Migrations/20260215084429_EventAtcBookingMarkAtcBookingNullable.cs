using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class EventAtcBookingMarkAtcBookingNullable : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_event_atc_position_booking_atc_booking_atc_booking_id",
                table: "event_atc_position_booking");

            migrationBuilder.AlterColumn<Guid>(
                name: "atc_booking_id",
                table: "event_atc_position_booking",
                type: "uuid",
                nullable: true,
                oldClrType: typeof(Guid),
                oldType: "uuid");

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

            migrationBuilder.AlterColumn<Guid>(
                name: "atc_booking_id",
                table: "event_atc_position_booking",
                type: "uuid",
                nullable: false,
                defaultValue: new Guid("00000000-0000-0000-0000-000000000000"),
                oldClrType: typeof(Guid),
                oldType: "uuid",
                oldNullable: true);

            migrationBuilder.AddForeignKey(
                name: "fk_event_atc_position_booking_atc_booking_atc_booking_id",
                table: "event_atc_position_booking",
                column: "atc_booking_id",
                principalTable: "atc_booking",
                principalColumn: "id",
                onDelete: ReferentialAction.Cascade);
        }
    }
}
