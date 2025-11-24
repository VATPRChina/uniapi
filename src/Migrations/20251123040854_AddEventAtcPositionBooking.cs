using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddEventAtcPositionBooking : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_event_atc_position_user_booking_user_id",
                table: "event_atc_position");

            migrationBuilder.DropIndex(
                name: "ix_event_atc_position_booking_user_id",
                table: "event_atc_position");

            migrationBuilder.DropColumn(
                name: "booked_at",
                table: "event_atc_position");

            migrationBuilder.DropColumn(
                name: "booking_user_id",
                table: "event_atc_position");

            migrationBuilder.CreateTable(
                name: "event_atc_position_booking",
                columns: table => new
                {
                    event_atc_position_id = table.Column<Guid>(type: "uuid", nullable: false),
                    user_id = table.Column<Guid>(type: "uuid", nullable: false),
                    created_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_event_atc_position_booking", x => x.event_atc_position_id);
                    table.ForeignKey(
                        name: "fk_event_atc_position_booking_event_atc_position_event_atc_pos",
                        column: x => x.event_atc_position_id,
                        principalTable: "event_atc_position",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_event_atc_position_booking_user_user_id",
                        column: x => x.user_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_event_atc_position_booking_user_id",
                table: "event_atc_position_booking",
                column: "user_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "event_atc_position_booking");

            migrationBuilder.AddColumn<DateTimeOffset>(
                name: "booked_at",
                table: "event_atc_position",
                type: "timestamp with time zone",
                nullable: true);

            migrationBuilder.AddColumn<Guid>(
                name: "booking_user_id",
                table: "event_atc_position",
                type: "uuid",
                nullable: true);

            migrationBuilder.CreateIndex(
                name: "ix_event_atc_position_booking_user_id",
                table: "event_atc_position",
                column: "booking_user_id");

            migrationBuilder.AddForeignKey(
                name: "fk_event_atc_position_user_booking_user_id",
                table: "event_atc_position",
                column: "booking_user_id",
                principalTable: "user",
                principalColumn: "id");
        }
    }
}
