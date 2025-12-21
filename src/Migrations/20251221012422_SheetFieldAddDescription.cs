using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class SheetFieldAddDescription : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AddColumn<string>(
                name: "description_en",
                table: "sheet_field",
                type: "text",
                nullable: true);

            migrationBuilder.AddColumn<string>(
                name: "description_zh",
                table: "sheet_field",
                type: "text",
                nullable: true);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropColumn(
                name: "description_en",
                table: "sheet_field");

            migrationBuilder.DropColumn(
                name: "description_zh",
                table: "sheet_field");
        }
    }
}
