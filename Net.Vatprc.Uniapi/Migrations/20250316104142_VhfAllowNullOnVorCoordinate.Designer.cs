﻿// <auto-generated />
using System;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Infrastructure;
using Microsoft.EntityFrameworkCore.Migrations;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;
using Net.Vatprc.Uniapi;
using Npgsql.EntityFrameworkCore.PostgreSQL.Metadata;

#nullable disable

namespace Net.Vatprc.Uniapi.Migrations
{
    [DbContext(typeof(VATPRCContext))]
    [Migration("20250316104142_VhfAllowNullOnVorCoordinate")]
    partial class VhfAllowNullOnVorCoordinate
    {
        /// <inheritdoc />
        protected override void BuildTargetModel(ModelBuilder modelBuilder)
        {
#pragma warning disable 612, 618
            modelBuilder
                .HasAnnotation("ProductVersion", "9.0.0")
                .HasAnnotation("Relational:MaxIdentifierLength", 63);

            NpgsqlModelBuilderExtensions.UseIdentityByDefaultColumns(modelBuilder);

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Acdm.Flight", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("Aircraft")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("aircraft");

                    b.Property<long>("Altitude")
                        .HasColumnType("bigint")
                        .HasColumnName("altitude");

                    b.Property<string>("Arrival")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("arrival");

                    b.Property<string>("ArrivalGate")
                        .HasColumnType("text")
                        .HasColumnName("arrival_gate");

                    b.Property<string>("ArrivalRunway")
                        .HasColumnType("text")
                        .HasColumnName("arrival_runway");

                    b.Property<string>("Callsign")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("callsign");

                    b.Property<string>("Cid")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("cid");

                    b.Property<long>("CruiseTas")
                        .HasColumnType("bigint")
                        .HasColumnName("cruise_tas");

                    b.Property<string>("Departure")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("departure");

                    b.Property<string>("DepartureGate")
                        .HasColumnType("text")
                        .HasColumnName("departure_gate");

                    b.Property<string>("DepartureRunway")
                        .HasColumnType("text")
                        .HasColumnName("departure_runway");

                    b.Property<string>("Equipment")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("equipment");

                    b.Property<DateTimeOffset?>("FinalizedAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("finalized_at");

                    b.Property<DateTimeOffset>("LastObservedAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("last_observed_at");

                    b.Property<double>("Latitude")
                        .HasColumnType("double precision")
                        .HasColumnName("latitude");

                    b.Property<double>("Longitude")
                        .HasColumnType("double precision")
                        .HasColumnName("longitude");

                    b.Property<string>("NavigationPerformance")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("navigation_performance");

                    b.Property<string>("RawRoute")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("raw_route");

                    b.Property<string>("State")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("state");

                    b.Property<string>("Transponder")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("transponder");

                    b.HasKey("Id")
                        .HasName("pk_flight");

                    b.HasIndex("Callsign")
                        .HasDatabaseName("ix_flight_callsign");

                    b.HasIndex("Cid")
                        .HasDatabaseName("ix_flight_cid");

                    b.ToTable("flight", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.DeviceAuthorization", b =>
                {
                    b.Property<Guid>("DeviceCode")
                        .HasColumnType("uuid")
                        .HasColumnName("device_code");

                    b.Property<string>("ClientId")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("client_id");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<DateTimeOffset>("ExpiresAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("expires_at");

                    b.Property<string>("UserCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("user_code");

                    b.Property<Guid?>("UserId")
                        .HasColumnType("uuid")
                        .HasColumnName("user_id");

                    b.HasKey("DeviceCode")
                        .HasName("pk_device_authorization");

                    b.HasIndex("UserCode")
                        .IsUnique()
                        .HasDatabaseName("ix_device_authorization_user_code");

                    b.HasIndex("UserId")
                        .HasDatabaseName("ix_device_authorization_user_id");

                    b.ToTable("device_authorization", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Event", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<string>("Description")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("description");

                    b.Property<DateTimeOffset>("EndAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("end_at");

                    b.Property<DateTimeOffset>("EndBookingAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("end_booking_at");

                    b.Property<string>("ImageUrl")
                        .HasColumnType("text")
                        .HasColumnName("image_url");

                    b.Property<DateTimeOffset>("StartAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("start_at");

                    b.Property<DateTimeOffset>("StartBookingAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("start_booking_at");

                    b.Property<string>("Title")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("title");

                    b.Property<DateTimeOffset>("UpdatedAt")
                        .ValueGeneratedOnAddOrUpdate()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("updated_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.HasKey("Id")
                        .HasName("pk_event");

                    b.ToTable("event", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventAirspace", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<string>("Description")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("description");

                    b.Property<Guid>("EventId")
                        .HasColumnType("uuid")
                        .HasColumnName("event_id");

                    b.PrimitiveCollection<string[]>("IcaoCodes")
                        .IsRequired()
                        .HasColumnType("text[]")
                        .HasColumnName("icao_codes");

                    b.Property<string>("Name")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("name");

                    b.Property<DateTimeOffset>("UpdatedAt")
                        .ValueGeneratedOnAddOrUpdate()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("updated_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.HasKey("Id")
                        .HasName("pk_event_airspace");

                    b.HasIndex("EventId")
                        .HasDatabaseName("ix_event_airspace_event_id");

                    b.ToTable("event_airspace", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventBooking", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<Guid>("EventSlotId")
                        .HasColumnType("uuid")
                        .HasColumnName("event_slot_id");

                    b.Property<DateTimeOffset>("UpdatedAt")
                        .ValueGeneratedOnAddOrUpdate()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("updated_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<Guid>("UserId")
                        .HasColumnType("uuid")
                        .HasColumnName("user_id");

                    b.HasKey("Id")
                        .HasName("pk_event_booking");

                    b.HasIndex("EventSlotId")
                        .IsUnique()
                        .HasDatabaseName("ix_event_booking_event_slot_id");

                    b.HasIndex("UserId")
                        .HasDatabaseName("ix_event_booking_user_id");

                    b.ToTable("event_booking", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventSlot", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("AircraftTypeIcao")
                        .HasColumnType("text")
                        .HasColumnName("aircraft_type_icao");

                    b.Property<string>("Callsign")
                        .HasColumnType("text")
                        .HasColumnName("callsign");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<DateTimeOffset>("EnterAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("enter_at");

                    b.Property<Guid>("EventAirspaceId")
                        .HasColumnType("uuid")
                        .HasColumnName("event_airspace_id");

                    b.Property<DateTimeOffset?>("LeaveAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("leave_at");

                    b.Property<DateTimeOffset>("UpdatedAt")
                        .ValueGeneratedOnAddOrUpdate()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("updated_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.HasKey("Id")
                        .HasName("pk_event_slot");

                    b.HasIndex("EventAirspaceId")
                        .HasDatabaseName("ix_event_slot_event_airspace_id");

                    b.ToTable("event_slot", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Airport", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<int>("Elevation")
                        .HasColumnType("integer")
                        .HasColumnName("elevation");

                    b.Property<string>("Identifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("identifier");

                    b.Property<double>("Latitude")
                        .HasColumnType("double precision")
                        .HasColumnName("latitude");

                    b.Property<double>("Longitude")
                        .HasColumnType("double precision")
                        .HasColumnName("longitude");

                    b.HasKey("Id")
                        .HasName("pk_airport");

                    b.HasIndex("Identifier")
                        .IsUnique()
                        .HasDatabaseName("ix_airport_identifier");

                    b.ToTable("airport", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.AirportGate", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<Guid>("AirportId")
                        .HasColumnType("uuid")
                        .HasColumnName("airport_id");

                    b.Property<string>("Identifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("identifier");

                    b.Property<double>("Latitude")
                        .HasColumnType("double precision")
                        .HasColumnName("latitude");

                    b.Property<double>("Longitude")
                        .HasColumnType("double precision")
                        .HasColumnName("longitude");

                    b.HasKey("Id")
                        .HasName("pk_airport_gate");

                    b.HasIndex("AirportId")
                        .HasDatabaseName("ix_airport_gate_airport_id");

                    b.ToTable("airport_gate", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.AirportPhysicalRunway", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<Guid>("AirportId")
                        .HasColumnType("uuid")
                        .HasColumnName("airport_id");

                    b.Property<Guid>("Runway1Id")
                        .HasColumnType("uuid")
                        .HasColumnName("runway1_id");

                    b.Property<Guid>("Runway2Id")
                        .HasColumnType("uuid")
                        .HasColumnName("runway2_id");

                    b.HasKey("Id")
                        .HasName("pk_airport_physical_runway");

                    b.HasIndex("AirportId")
                        .HasDatabaseName("ix_airport_physical_runway_airport_id");

                    b.HasIndex("Runway1Id")
                        .IsUnique()
                        .HasDatabaseName("ix_airport_physical_runway_runway1id");

                    b.HasIndex("Runway2Id")
                        .IsUnique()
                        .HasDatabaseName("ix_airport_physical_runway_runway2id");

                    b.ToTable("airport_physical_runway", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Airway", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("Identifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("identifier");

                    b.HasKey("Id")
                        .HasName("pk_airway");

                    b.ToTable("airway", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.AirwayFix", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<Guid>("AirwayId")
                        .HasColumnType("uuid")
                        .HasColumnName("airway_id");

                    b.Property<string>("DescriptionCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("description_code");

                    b.Property<string>("FixIcaoCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("fix_icao_code");

                    b.Property<string>("FixIdentifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("fix_identifier");

                    b.Property<long>("SequenceNumber")
                        .HasColumnType("bigint")
                        .HasColumnName("sequence_number");

                    b.HasKey("Id")
                        .HasName("pk_airway_fix");

                    b.HasIndex("AirwayId")
                        .HasDatabaseName("ix_airway_fix_airway_id");

                    b.ToTable("airway_fix", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.NdbNavaid", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("AirportIcaoIdent")
                        .HasColumnType("text")
                        .HasColumnName("airport_icao_ident");

                    b.Property<string>("IcaoCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("icao_code");

                    b.Property<string>("Identifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("identifier");

                    b.Property<double>("Latitude")
                        .HasColumnType("double precision")
                        .HasColumnName("latitude");

                    b.Property<double>("Longitude")
                        .HasColumnType("double precision")
                        .HasColumnName("longitude");

                    b.Property<string>("SectionCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("section_code");

                    b.HasKey("Id")
                        .HasName("pk_ndb_navaid");

                    b.ToTable("ndb_navaid", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.PreferredRoute", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("Arrival")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("arrival");

                    b.Property<string>("Departure")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("departure");

                    b.Property<string>("RawRoute")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("raw_route");

                    b.HasKey("Id")
                        .HasName("pk_preferred_route");

                    b.ToTable("preferred_route", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Procedure", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<Guid>("AirportId")
                        .HasColumnType("uuid")
                        .HasColumnName("airport_id");

                    b.Property<string>("Identifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("identifier");

                    b.Property<char>("SubsectionCode")
                        .HasColumnType("character(1)")
                        .HasColumnName("subsection_code");

                    b.HasKey("Id")
                        .HasName("pk_procedure");

                    b.HasIndex("AirportId")
                        .HasDatabaseName("ix_procedure_airport_id");

                    b.ToTable("procedure", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Runway", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<Guid>("AirportId")
                        .HasColumnType("uuid")
                        .HasColumnName("airport_id");

                    b.Property<string>("Identifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("identifier");

                    b.Property<double>("Latitude")
                        .HasColumnType("double precision")
                        .HasColumnName("latitude");

                    b.Property<double>("Longitude")
                        .HasColumnType("double precision")
                        .HasColumnName("longitude");

                    b.HasKey("Id")
                        .HasName("pk_runway");

                    b.HasIndex("AirportId")
                        .HasDatabaseName("ix_runway_airport_id");

                    b.ToTable("runway", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.VhfNavaid", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("DmeIdentifier")
                        .HasColumnType("text")
                        .HasColumnName("dme_identifier");

                    b.Property<double?>("DmeLatitude")
                        .HasColumnType("double precision")
                        .HasColumnName("dme_latitude");

                    b.Property<double?>("DmeLongitude")
                        .HasColumnType("double precision")
                        .HasColumnName("dme_longitude");

                    b.Property<string>("IcaoCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("icao_code");

                    b.Property<string>("VorIdentifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("vor_identifier");

                    b.Property<double?>("VorLatitude")
                        .HasColumnType("double precision")
                        .HasColumnName("vor_latitude");

                    b.Property<double?>("VorLongitude")
                        .HasColumnType("double precision")
                        .HasColumnName("vor_longitude");

                    b.HasKey("Id")
                        .HasName("pk_vhf_navaid");

                    b.ToTable("vhf_navaid", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Waypoint", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("IcaoCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("icao_code");

                    b.Property<string>("Identifier")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("identifier");

                    b.Property<double>("Latitude")
                        .HasColumnType("double precision")
                        .HasColumnName("latitude");

                    b.Property<double>("Longitude")
                        .HasColumnType("double precision")
                        .HasColumnName("longitude");

                    b.Property<string>("RegionCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("region_code");

                    b.Property<string>("SectionCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("section_code");

                    b.HasKey("Id")
                        .HasName("pk_waypoint");

                    b.ToTable("waypoint", "navdata");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.RefreshToken", b =>
                {
                    b.Property<Guid>("Token")
                        .HasColumnType("uuid")
                        .HasColumnName("token");

                    b.Property<Guid?>("AuthzCode")
                        .HasColumnType("uuid")
                        .HasColumnName("code");

                    b.Property<string>("ClientId")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("client_id");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<DateTimeOffset>("ExpiresIn")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("expires_in");

                    b.Property<Guid>("UserId")
                        .HasColumnType("uuid")
                        .HasColumnName("user_id");

                    b.Property<DateTimeOffset>("UserUpdatedAt")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("user_updated_at");

                    b.HasKey("Token")
                        .HasName("pk_session");

                    b.HasIndex("UserId")
                        .HasDatabaseName("ix_session_user_id");

                    b.ToTable("session", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.User", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<string>("Cid")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("cid");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<string>("Email")
                        .HasColumnType("text")
                        .HasColumnName("email");

                    b.Property<string>("FullName")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("full_name");

                    b.PrimitiveCollection<string[]>("Roles")
                        .IsRequired()
                        .HasColumnType("text[]")
                        .HasColumnName("roles");

                    b.Property<DateTimeOffset>("UpdatedAt")
                        .ValueGeneratedOnAddOrUpdate()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("updated_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.HasKey("Id")
                        .HasName("pk_user");

                    b.HasIndex("Cid")
                        .IsUnique()
                        .HasDatabaseName("ix_user_cid");

                    b.HasIndex("Email")
                        .IsUnique()
                        .HasDatabaseName("ix_user_email");

                    b.ToTable("user", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.DeviceAuthorization", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.User", "User")
                        .WithMany()
                        .HasForeignKey("UserId")
                        .HasConstraintName("fk_device_authorization_user_user_id");

                    b.Navigation("User");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventAirspace", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Event", "Event")
                        .WithMany("Airspaces")
                        .HasForeignKey("EventId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_event_airspace_event_event_id");

                    b.Navigation("Event");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventBooking", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.EventSlot", "EventSlot")
                        .WithOne("Booking")
                        .HasForeignKey("Net.Vatprc.Uniapi.Models.EventBooking", "EventSlotId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_event_booking_event_slot_event_slot_id");

                    b.HasOne("Net.Vatprc.Uniapi.Models.User", "User")
                        .WithMany()
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_event_booking_user_user_id");

                    b.Navigation("EventSlot");

                    b.Navigation("User");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventSlot", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.EventAirspace", "EventAirspace")
                        .WithMany("Slots")
                        .HasForeignKey("EventAirspaceId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_event_slot_event_airspace_event_airspace_id");

                    b.Navigation("EventAirspace");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.AirportGate", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Navdata.Airport", "Airport")
                        .WithMany("Gates")
                        .HasForeignKey("AirportId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_airport_gate_airport_airport_id");

                    b.Navigation("Airport");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.AirportPhysicalRunway", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Navdata.Airport", "Airport")
                        .WithMany("PhysicalRunways")
                        .HasForeignKey("AirportId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_airport_physical_runway_airport_airport_id");

                    b.HasOne("Net.Vatprc.Uniapi.Models.Navdata.Runway", "Runway1")
                        .WithOne()
                        .HasForeignKey("Net.Vatprc.Uniapi.Models.Navdata.AirportPhysicalRunway", "Runway1Id")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_airport_physical_runway_runway_runway1id");

                    b.HasOne("Net.Vatprc.Uniapi.Models.Navdata.Runway", "Runway2")
                        .WithOne()
                        .HasForeignKey("Net.Vatprc.Uniapi.Models.Navdata.AirportPhysicalRunway", "Runway2Id")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_airport_physical_runway_runway_runway2id");

                    b.Navigation("Airport");

                    b.Navigation("Runway1");

                    b.Navigation("Runway2");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.AirwayFix", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Navdata.Airway", "Airway")
                        .WithMany("Fixes")
                        .HasForeignKey("AirwayId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_airway_fix_airway_airway_id");

                    b.Navigation("Airway");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Procedure", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Navdata.Airport", "Airport")
                        .WithMany("Procedures")
                        .HasForeignKey("AirportId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_procedure_airport_airport_id");

                    b.Navigation("Airport");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Runway", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Navdata.Airport", "Airport")
                        .WithMany("Runways")
                        .HasForeignKey("AirportId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_runway_airport_airport_id");

                    b.Navigation("Airport");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.RefreshToken", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.User", "User")
                        .WithMany("Sessions")
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_session_user_user_id");

                    b.Navigation("User");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Event", b =>
                {
                    b.Navigation("Airspaces");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventAirspace", b =>
                {
                    b.Navigation("Slots");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.EventSlot", b =>
                {
                    b.Navigation("Booking");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Airport", b =>
                {
                    b.Navigation("Gates");

                    b.Navigation("PhysicalRunways");

                    b.Navigation("Procedures");

                    b.Navigation("Runways");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Navdata.Airway", b =>
                {
                    b.Navigation("Fixes");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.User", b =>
                {
                    b.Navigation("Sessions");
                });
#pragma warning restore 612, 618
        }
    }
}
