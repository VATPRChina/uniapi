using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class SheetFieldAddId : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_sheet_filing_answer_sheet_field_sheet_id_field_sequence",
                table: "sheet_filing_answer");

            migrationBuilder.DropPrimaryKey(
                name: "pk_sheet_filing_answer",
                table: "sheet_filing_answer");

            migrationBuilder.DropPrimaryKey(
                name: "pk_sheet_field",
                table: "sheet_field");

            migrationBuilder.DropColumn(
                name: "field_sequence",
                table: "sheet_filing_answer");

            migrationBuilder.AddColumn<string>(
                name: "field_id",
                table: "sheet_filing_answer",
                type: "text",
                nullable: false,
                defaultValue: "");

            migrationBuilder.AddColumn<string>(
                name: "id",
                table: "sheet_field",
                type: "text",
                nullable: false,
                defaultValue: "");

            migrationBuilder.AddColumn<bool>(
                name: "is_deleted",
                table: "sheet_field",
                type: "boolean",
                nullable: false,
                defaultValue: false);

            migrationBuilder.AddPrimaryKey(
                name: "pk_sheet_filing_answer",
                table: "sheet_filing_answer",
                columns: new[] { "sheet_id", "field_id", "filing_id" });

            migrationBuilder.AddPrimaryKey(
                name: "pk_sheet_field",
                table: "sheet_field",
                columns: new[] { "sheet_id", "id" });

            migrationBuilder.AddForeignKey(
                name: "fk_sheet_filing_answer_sheet_field_sheet_id_field_id",
                table: "sheet_filing_answer",
                columns: new[] { "sheet_id", "field_id" },
                principalTable: "sheet_field",
                principalColumns: new[] { "sheet_id", "id" },
                onDelete: ReferentialAction.Cascade);
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropForeignKey(
                name: "fk_sheet_filing_answer_sheet_field_sheet_id_field_id",
                table: "sheet_filing_answer");

            migrationBuilder.DropPrimaryKey(
                name: "pk_sheet_filing_answer",
                table: "sheet_filing_answer");

            migrationBuilder.DropPrimaryKey(
                name: "pk_sheet_field",
                table: "sheet_field");

            migrationBuilder.DropColumn(
                name: "field_id",
                table: "sheet_filing_answer");

            migrationBuilder.DropColumn(
                name: "id",
                table: "sheet_field");

            migrationBuilder.DropColumn(
                name: "is_deleted",
                table: "sheet_field");

            migrationBuilder.AddColumn<long>(
                name: "field_sequence",
                table: "sheet_filing_answer",
                type: "bigint",
                nullable: false,
                defaultValue: 0L);

            migrationBuilder.AddPrimaryKey(
                name: "pk_sheet_filing_answer",
                table: "sheet_filing_answer",
                columns: new[] { "sheet_id", "field_sequence", "filing_id" });

            migrationBuilder.AddPrimaryKey(
                name: "pk_sheet_field",
                table: "sheet_field",
                columns: new[] { "sheet_id", "sequence" });

            migrationBuilder.AddForeignKey(
                name: "fk_sheet_filing_answer_sheet_field_sheet_id_field_sequence",
                table: "sheet_filing_answer",
                columns: new[] { "sheet_id", "field_sequence" },
                principalTable: "sheet_field",
                principalColumns: new[] { "sheet_id", "sequence" },
                onDelete: ReferentialAction.Cascade);
        }
    }
}
