using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddUserAtcStatus : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<Guid>(
                name: "atc_status_user_id",
                table: "user",
                type: "uuid",
                nullable: true);

            migrationBuilder.CreateTable(
                name: "user_atc_status",
                columns: table => new
                {
                    user_id = table.Column<Guid>(type: "uuid", nullable: false),
                    is_visiting = table.Column<bool>(type: "boolean", nullable: false),
                    is_absent = table.Column<bool>(type: "boolean", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_user_atc_status", x => x.user_id);
                    table.ForeignKey(
                        name: "fk_user_atc_status_user_user_id",
                        column: x => x.user_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_user_atc_status_user_id",
                table: "user",
                column: "atc_status_user_id");

            migrationBuilder.AddForeignKey(
                name: "fk_user_user_atc_status_atc_status_user_id",
                table: "user",
                column: "atc_status_user_id",
                principalTable: "user_atc_status",
                principalColumn: "user_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_user_user_atc_status_atc_status_user_id",
                table: "user");

            migrationBuilder.DropTable(
                name: "user_atc_status");

            migrationBuilder.DropIndex(
                name: "ix_user_atc_status_user_id",
                table: "user");

            migrationBuilder.DropColumn(
                name: "atc_status_user_id",
                table: "user");
        }
    }
}
