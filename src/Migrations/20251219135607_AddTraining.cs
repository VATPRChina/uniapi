using System;
using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class AddTraining : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.CreateTable(
                name: "training",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    trainer_id = table.Column<Guid>(type: "uuid", nullable: false),
                    trainee_id = table.Column<Guid>(type: "uuid", nullable: false),
                    record_sheet_filing_id = table.Column<Guid>(type: "uuid", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_training", x => x.id);
                    table.ForeignKey(
                        name: "fk_training_sheet_filing_record_sheet_filing_id",
                        column: x => x.record_sheet_filing_id,
                        principalTable: "sheet_filing",
                        principalColumn: "id");
                    table.ForeignKey(
                        name: "fk_training_user_trainee_id",
                        column: x => x.trainee_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_training_user_trainer_id",
                        column: x => x.trainer_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "training_application",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    trainee_id = table.Column<Guid>(type: "uuid", nullable: false),
                    name = table.Column<string>(type: "text", nullable: false),
                    train_id = table.Column<Guid>(type: "uuid", nullable: true)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_training_application", x => x.id);
                    table.ForeignKey(
                        name: "fk_training_application_training_train_id",
                        column: x => x.train_id,
                        principalTable: "training",
                        principalColumn: "id");
                    table.ForeignKey(
                        name: "fk_training_application_user_trainee_id",
                        column: x => x.trainee_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateTable(
                name: "training_application_response",
                columns: table => new
                {
                    id = table.Column<Guid>(type: "uuid", nullable: false),
                    application_id = table.Column<Guid>(type: "uuid", nullable: false),
                    trainer_id = table.Column<Guid>(type: "uuid", nullable: false),
                    is_accepted = table.Column<bool>(type: "boolean", nullable: false),
                    comment = table.Column<string>(type: "text", nullable: false),
                    created_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false),
                    updated_at = table.Column<DateTimeOffset>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("pk_training_application_response", x => x.id);
                    table.ForeignKey(
                        name: "fk_training_application_response_training_application_applicat",
                        column: x => x.application_id,
                        principalTable: "training_application",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                    table.ForeignKey(
                        name: "fk_training_application_response_user_trainer_id",
                        column: x => x.trainer_id,
                        principalTable: "user",
                        principalColumn: "id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "ix_training_record_sheet_filing_id",
                table: "training",
                column: "record_sheet_filing_id");

            migrationBuilder.CreateIndex(
                name: "ix_training_trainee_id",
                table: "training",
                column: "trainee_id");

            migrationBuilder.CreateIndex(
                name: "ix_training_trainer_id",
                table: "training",
                column: "trainer_id");

            migrationBuilder.CreateIndex(
                name: "ix_training_application_train_id",
                table: "training_application",
                column: "train_id",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "ix_training_application_trainee_id",
                table: "training_application",
                column: "trainee_id");

            migrationBuilder.CreateIndex(
                name: "ix_training_application_response_application_id",
                table: "training_application_response",
                column: "application_id");

            migrationBuilder.CreateIndex(
                name: "ix_training_application_response_trainer_id",
                table: "training_application_response",
                column: "trainer_id");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "training_application_response");

            migrationBuilder.DropTable(
                name: "training_application");

            migrationBuilder.DropTable(
                name: "training");
        }
    }
}
