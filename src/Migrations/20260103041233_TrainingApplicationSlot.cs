using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class TrainingApplicationSlot : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "is_accepted",
                table: "training_application_response");

            migrationBuilder.DropColumn(
                name: "end_at",
                table: "training_application");

            migrationBuilder.DropColumn(
                name: "start_at",
                table: "training_application");

            migrationBuilder.AddColumn<Guid>(
                name: "slot_id",
                table: "training_application_response",
                type: "uuid",
                nullable: true);

            migrationBuilder.AddColumn<DateTimeOffset>(
                name: "deleted_at",
                table: "training_application",
                type: "timestamp with time zone",
                nullable: true);

            migrationBuilder.CreateTable(
                name: "training_application_slot",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    application_id = table.Column<Guid>(type: "uuid", nullable: false),
                    start_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false),
                    end_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_training_application_slot", x => x.id);
                    table.ForeignKey(
                        name: "fk_training_application_slot_training_application_application_",
                        column: x => x.application_id,
                        principalTable: "training_application",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_training_application_response_slot_id",
                table: "training_application_response",
                column: "slot_id",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "ix_training_application_slot_application_id",
                table: "training_application_slot",
                column: "application_id");

            migrationBuilder.AddForeignKey(
                name: "fk_training_application_response_training_application_slot_slo",
                table: "training_application_response",
                column: "slot_id",
                principalTable: "training_application_slot",
                principalColumn: "id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_training_application_response_training_application_slot_slo",
                table: "training_application_response");

            migrationBuilder.DropTable(
                name: "training_application_slot");

            migrationBuilder.DropIndex(
                name: "ix_training_application_response_slot_id",
                table: "training_application_response");

            migrationBuilder.DropColumn(
                name: "slot_id",
                table: "training_application_response");

            migrationBuilder.DropColumn(
                name: "deleted_at",
                table: "training_application");

            migrationBuilder.AddColumn<bool>(
                name: "is_accepted",
                table: "training_application_response",
                type: "boolean",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddColumn<DateTimeOffset>(
                name: "end_at",
                table: "training_application",
                type: "timestamp with time zone",
                nullable: false,
                defaultValue: new DateTimeOffset(new DateTime(1, 1, 1, 0, 0, 0, 0, DateTimeKind.Unspecified), new TimeSpan(0, 0, 0, 0, 0)));

            migrationBuilder.AddColumn<DateTimeOffset>(
                name: "start_at",
                table: "training_application",
                type: "timestamp with time zone",
                nullable: false,
                defaultValue: new DateTimeOffset(new DateTime(1, 1, 1, 0, 0, 0, 0, DateTimeKind.Unspecified), new TimeSpan(0, 0, 0, 0, 0)));
        }
    }
}
