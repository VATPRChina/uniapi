using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddAtcApplication : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterColumn<string>(
                name: "name_en",
                table: "sheet_field",
                type: "text",
                nullable: true,
                oldClrType: typeof(string),
                oldType: "text");

            migrationBuilder.CreateTable(
                name: "atc_application",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    user_id = table.Column<Guid>(type: "uuid", nullable: false),
                    application_filing_id = table.Column<Guid>(type: "uuid", nullable: false),
                    review_filing_id = table.Column<Guid>(type: "uuid", nullable: true),
                    applied_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_atc_application", x => x.id);
                    table.ForeignKey(
                        name: "fk_atc_application_sheet_filing_application_filing_id",
                        column: x => x.application_filing_id,
                        principalTable: "sheet_filing",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_atc_application_sheet_filing_review_filing_id",
                        column: x => x.review_filing_id,
                        principalTable: "sheet_filing",
                        principalColumn: "id");
                    table.ForeignKey(
                        name: "fk_atc_application_user_user_id",
                        column: x => x.user_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_atc_application_application_filing_id",
                table: "atc_application",
                column: "application_filing_id");

            migrationBuilder.CreateIndex(
                name: "ix_atc_application_review_filing_id",
                table: "atc_application",
                column: "review_filing_id");

            migrationBuilder.CreateIndex(
                name: "ix_atc_application_user_id",
                table: "atc_application",
                column: "user_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "atc_application");

            migrationBuilder.AlterColumn<string>(
                name: "name_en",
                table: "sheet_field",
                type: "text",
                nullable: false,
                defaultValue: "",
                oldClrType: typeof(string),
                oldType: "text",
                oldNullable: true);
        }
    }
}
