using Microsoft.EntityFrameworkCore.Migrations;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    /// <inheritdoc />
    public partial class VhfAllowNullOnVorCoordinate : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterColumn<double>(
                name: "vor_longitude",
                schema: "navdata",
                table: "vhf_navaid",
                type: "double precision",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "double precision");

            migrationBuilder.AlterColumn<double>(
                name: "vor_latitude",
                schema: "navdata",
                table: "vhf_navaid",
                type: "double precision",
                nullable: true,
                oldClrType: typeof(double),
                oldType: "double precision");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterColumn<double>(
                name: "vor_longitude",
                schema: "navdata",
                table: "vhf_navaid",
                type: "double precision",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(double),
                oldType: "double precision",
                oldNullable: true);

            migrationBuilder.AlterColumn<double>(
                name: "vor_latitude",
                schema: "navdata",
                table: "vhf_navaid",
                type: "double precision",
                nullable: false,
                defaultValue: 0.0,
                oldClrType: typeof(double),
                oldType: "double precision",
                oldNullable: true);
        }
    }
}
