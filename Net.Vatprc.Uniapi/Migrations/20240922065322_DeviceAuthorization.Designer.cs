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
    [Migration("20240922065322_DeviceAuthorization")]
    partial class DeviceAuthorization
    {
        /// <inheritdoc />
        protected override void BuildTargetModel(ModelBuilder modelBuilder)
        {
#pragma warning disable 612, 618
            modelBuilder
                .HasAnnotation("ProductVersion", "8.0.2")
                .HasAnnotation("Relational:MaxIdentifierLength", 63);

            NpgsqlModelBuilderExtensions.UseIdentityByDefaultColumns(modelBuilder);

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.DeviceAuthorization", b =>
                {
                    b.Property<Guid>("DeviceCode")
                        .HasColumnType("uuid")
                        .HasColumnName("device_code");

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

                    b.Property<string[]>("IcaoCodes")
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

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Notam", b =>
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

                    b.Property<DateTimeOffset>("EffectiveFrom")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("effective_from");

                    b.Property<DateTimeOffset>("ExpireAfter")
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("expire_after");

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
                        .HasName("pk_notam");

                    b.ToTable("notam", (string)null);
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.NotamBinding", b =>
                {
                    b.Property<Guid>("Id")
                        .HasColumnType("uuid")
                        .HasColumnName("id");

                    b.Property<DateTimeOffset>("CreatedAt")
                        .ValueGeneratedOnAdd()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("created_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.Property<string>("Discriminator")
                        .IsRequired()
                        .HasMaxLength(34)
                        .HasColumnType("character varying(34)")
                        .HasColumnName("discriminator");

                    b.Property<Guid>("NotamId")
                        .HasColumnType("uuid")
                        .HasColumnName("notam_id");

                    b.Property<DateTimeOffset>("UpdatedAt")
                        .ValueGeneratedOnAddOrUpdate()
                        .HasColumnType("timestamp with time zone")
                        .HasColumnName("updated_at")
                        .HasDefaultValueSql("CURRENT_TIMESTAMP");

                    b.HasKey("Id")
                        .HasName("pk_notam_binding");

                    b.HasIndex("NotamId")
                        .HasDatabaseName("ix_notam_binding_notam_id");

                    b.ToTable("notam_binding", (string)null);

                    b.HasDiscriminator<string>("Discriminator").HasValue("NotamBinding");

                    b.UseTphMappingStrategy();
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Session", b =>
                {
                    b.Property<Guid>("Token")
                        .HasColumnType("uuid")
                        .HasColumnName("token");

                    b.Property<Guid?>("Code")
                        .HasColumnType("uuid")
                        .HasColumnName("code");

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

                    b.Property<string[]>("Roles")
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

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.NotamBindingEvent", b =>
                {
                    b.HasBaseType("Net.Vatprc.Uniapi.Models.NotamBinding");

                    b.Property<Guid>("EventId")
                        .HasColumnType("uuid")
                        .HasColumnName("event_id");

                    b.HasIndex("EventId")
                        .HasDatabaseName("ix_notam_binding_event_id");

                    b.ToTable("notam_binding", (string)null);

                    b.HasDiscriminator().HasValue("NotamBindingEvent");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.NotamBindingEventAirspace", b =>
                {
                    b.HasBaseType("Net.Vatprc.Uniapi.Models.NotamBinding");

                    b.Property<Guid>("EventAirspaceId")
                        .HasColumnType("uuid")
                        .HasColumnName("event_airspace_id");

                    b.HasIndex("EventAirspaceId")
                        .HasDatabaseName("ix_notam_binding_event_airspace_id");

                    b.ToTable("notam_binding", (string)null);

                    b.HasDiscriminator().HasValue("NotamBindingEventAirspace");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.NotamBindingIcaoCode", b =>
                {
                    b.HasBaseType("Net.Vatprc.Uniapi.Models.NotamBinding");

                    b.Property<string>("IcaoCode")
                        .IsRequired()
                        .HasColumnType("text")
                        .HasColumnName("icao_code");

                    b.HasIndex("IcaoCode")
                        .HasDatabaseName("ix_notam_binding_icao_code");

                    b.ToTable("notam_binding", (string)null);

                    b.HasDiscriminator().HasValue("NotamBindingIcaoCode");
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

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.NotamBinding", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Notam", "Notam")
                        .WithMany("Bindings")
                        .HasForeignKey("NotamId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_notam_binding_notam_notam_id");

                    b.Navigation("Notam");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Session", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.User", "User")
                        .WithMany("Sessions")
                        .HasForeignKey("UserId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_session_user_user_id");

                    b.Navigation("User");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.NotamBindingEvent", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.Event", "Event")
                        .WithMany()
                        .HasForeignKey("EventId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_notam_binding_event_event_id");

                    b.Navigation("Event");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.NotamBindingEventAirspace", b =>
                {
                    b.HasOne("Net.Vatprc.Uniapi.Models.EventAirspace", "EventAirspace")
                        .WithMany()
                        .HasForeignKey("EventAirspaceId")
                        .OnDelete(DeleteBehavior.Cascade)
                        .IsRequired()
                        .HasConstraintName("fk_notam_binding_event_airspace_event_airspace_id");

                    b.Navigation("EventAirspace");
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

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.Notam", b =>
                {
                    b.Navigation("Bindings");
                });

            modelBuilder.Entity("Net.Vatprc.Uniapi.Models.User", b =>
                {
                    b.Navigation("Sessions");
                });
#pragma warning restore 612, 618
        }
    }
}
