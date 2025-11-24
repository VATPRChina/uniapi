using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddSheet : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "sheet",
                columns: table => new
                {
                    id = table.Column<string>(type: "text", nullable: false),
                    name = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_sheet", x => x.id);
                });

            migrationBuilder.CreateTable(
                name: "sheet_field",
                columns: table => new
                {
                    sheet_id = table.Column<string>(type: "text", nullable: false),
                    sequence = table.Column<long>(type: "bigint", nullable: false),
                    name_zh = table.Column<string>(type: "text", nullable: false),
                    name_en = table.Column<string>(type: "text", nullable: false),
                    kind = table.Column<int>(type: "integer", nullable: false),
                    single_choice_options = table.Column<string[]>(type: "text[]", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_sheet_field", x => new { x.sheet_id, x.sequence });
                    table.ForeignKey(
                        name: "fk_sheet_field_sheet_sheet_id",
                        column: x => x.sheet_id,
                        principalTable: "sheet",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "sheet_filing",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    sheet_id = table.Column<string>(type: "text", nullable: false),
                    user_id = table.Column<Guid>(type: "uuid", nullable: false),
                    filed_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_sheet_filing", x => x.id);
                    table.ForeignKey(
                        name: "fk_sheet_filing_sheet_sheet_id",
                        column: x => x.sheet_id,
                        principalTable: "sheet",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_sheet_filing_user_user_id",
                        column: x => x.user_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "sheet_filing_answer",
                columns: table => new
                {
                    sheet_id = table.Column<string>(type: "text", nullable: false),
                    field_sequence = table.Column<long>(type: "bigint", nullable: false),
                    filing_id = table.Column<Guid>(type: "uuid", nullable: false),
                    answer = table.Column<string>(type: "text", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_sheet_filing_answer", x => new { x.sheet_id, x.field_sequence, x.filing_id });
                    table.ForeignKey(
                        name: "fk_sheet_filing_answer_sheet_field_sheet_id_field_sequence",
                        columns: x => new { x.sheet_id, x.field_sequence },
                        principalTable: "sheet_field",
                        principalColumns: new[] { "sheet_id", "sequence" },
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_sheet_filing_answer_sheet_filing_filing_id",
                        column: x => x.filing_id,
                        principalTable: "sheet_filing",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_sheet_filing_sheet_id",
                table: "sheet_filing",
                column: "sheet_id");

            migrationBuilder.CreateIndex(
                name: "ix_sheet_filing_user_id",
                table: "sheet_filing",
                column: "user_id");

            migrationBuilder.CreateIndex(
                name: "ix_sheet_filing_answer_filing_id",
                table: "sheet_filing_answer",
                column: "filing_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "sheet_filing_answer");

            migrationBuilder.DropTable(
                name: "sheet_field");

            migrationBuilder.DropTable(
                name: "sheet_filing");

            migrationBuilder.DropTable(
                name: "sheet");
        }
    }
}
