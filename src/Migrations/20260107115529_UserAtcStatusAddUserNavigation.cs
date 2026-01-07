using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class UserAtcStatusAddUserNavigation : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_user_user_atc_status_atc_status_user_id",
                table: "user");

            migrationBuilder.DropIndex(
                name: "ix_user_atc_status_user_id",
                table: "user");

            migrationBuilder.DropColumn(
                name: "atc_status_user_id",
                table: "user");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<Guid>(
                name: "atc_status_user_id",
                table: "user",
                type: "uuid",
                nullable: true);

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
    }
}
