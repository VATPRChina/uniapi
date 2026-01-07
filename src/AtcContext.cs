using System;
using System.Collections.Generic;
using Microsoft.EntityFrameworkCore;

namespace Net.Vatprc.AtcApi;

public partial class AtcContext : DbContext
{
    public AtcContext()
    {
    }

    public AtcContext(DbContextOptions<AtcContext> options)
        : base(options)
    {
    }

    public virtual DbSet<Application> Applications { get; set; }

    public virtual DbSet<ApplicationsInterview> ApplicationsInterviews { get; set; }

    public virtual DbSet<ApplicationsMeta> ApplicationsMetas { get; set; }

    public virtual DbSet<Event> Events { get; set; }

    public virtual DbSet<EventsPosition> EventsPositions { get; set; }

    public virtual DbSet<EventsPositionsBooking> EventsPositionsBookings { get; set; }

    public virtual DbSet<FailedJob> FailedJobs { get; set; }

    public virtual DbSet<Job> Jobs { get; set; }

    public virtual DbSet<Migration> Migrations { get; set; }

    public virtual DbSet<Permission> Permissions { get; set; }

    public virtual DbSet<Role> Roles { get; set; }

    public virtual DbSet<RolesInheritance> RolesInheritances { get; set; }

    public virtual DbSet<RolesPermission> RolesPermissions { get; set; }

    public virtual DbSet<Schedule> Schedules { get; set; }

    public virtual DbSet<Train> Trains { get; set; }

    public virtual DbSet<TrainComment> TrainComments { get; set; }

    public virtual DbSet<TrainRequest> TrainRequests { get; set; }

    public virtual DbSet<TrainRequestsPeriod> TrainRequestsPeriods { get; set; }

    public virtual DbSet<TrainsBooking> TrainsBookings { get; set; }

    public virtual DbSet<User> Users { get; set; }

    public virtual DbSet<UsersRole> UsersRoles { get; set; }

    public virtual DbSet<UsersToken> UsersTokens { get; set; }

    protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
#warning To protect potentially sensitive information in your connection string, you should move it out of source code. You can avoid scaffolding the connection string by using the Name= syntax to read it from configuration - see https://go.microsoft.com/fwlink/?linkid=2131148. For more guidance on storing connection strings, see https://go.microsoft.com/fwlink/?LinkId=723263.
        => optionsBuilder.UseNpgsql("Host=localhost;Database=vatprc_atc;Username=xfoxfu");

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<Application>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632005_primary");

            entity.ToTable("applications", "vatprc_atc");

            entity.HasIndex(e => e.Applicant, "idx_632005_applications_applicant_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('applications_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Accepted).HasColumnName("accepted");
            entity.Property(e => e.Applicant).HasColumnName("applicant");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.LastReviewedAt).HasColumnName("last_reviewed_at");
            entity.Property(e => e.ProcessedAt).HasColumnName("processed_at");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
        });

        modelBuilder.Entity<ApplicationsInterview>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632016_primary");

            entity.ToTable("applications_interviews", "vatprc_atc");

            entity.HasIndex(e => e.ApplicationId, "idx_632016_applications_interviews_application_id_foreign");

            entity.HasIndex(e => e.Interviewer, "idx_632016_applications_interviews_interviewer_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('applications_interviews_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.ApplicationId).HasColumnName("application_id");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Interviewer).HasColumnName("interviewer");
            entity.Property(e => e.ScheduleTime).HasColumnName("schedule_time");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");

            entity.HasOne(d => d.InterviewerNavigation).WithMany(p => p.ApplicationsInterviews)
                .HasForeignKey(d => d.Interviewer)
                .OnDelete(DeleteBehavior.ClientSetNull)
                .HasConstraintName("applications_interviews_interviewer_foreign");
        });

        modelBuilder.Entity<ApplicationsMeta>(entity =>
        {
            entity.HasKey(e => e.ApplicationId).HasName("idx_632026_primary");

            entity.ToTable("applications_metas", "vatprc_atc");

            entity.Property(e => e.ApplicationId).HasColumnName("application_id");
            entity.Property(e => e.Age).HasColumnName("age");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.EnglishLevel).HasColumnName("english_level");
            entity.Property(e => e.Expectation).HasColumnName("expectation");
            entity.Property(e => e.Location).HasColumnName("location");
            entity.Property(e => e.Occupation).HasColumnName("occupation");
            entity.Property(e => e.PreviousAtc).HasColumnName("previous_atc");
            entity.Property(e => e.Remark).HasColumnName("remark");
            entity.Property(e => e.SelfIntroduction).HasColumnName("self_introduction");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
            entity.Property(e => e.WeeklyHours).HasColumnName("weekly_hours");
        });

        modelBuilder.Entity<Event>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632041_primary");

            entity.ToTable("events", "vatprc_atc");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('events_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.BannerUrl).HasColumnName("banner_url");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Finish).HasColumnName("finish");
            entity.Property(e => e.Open).HasColumnName("open");
            entity.Property(e => e.OpenFrom).HasColumnName("open_from");
            entity.Property(e => e.PublishFrom).HasColumnName("publish_from");
            entity.Property(e => e.Published).HasColumnName("published");
            entity.Property(e => e.Remark).HasColumnName("remark");
            entity.Property(e => e.Start).HasColumnName("start");
            entity.Property(e => e.Title).HasColumnName("title");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
            entity.Property(e => e.Url).HasColumnName("url");
        });

        modelBuilder.Entity<EventsPosition>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632056_primary");

            entity.ToTable("events_positions", "vatprc_atc");

            entity.HasIndex(e => e.EventId, "idx_632056_events_positions_event_id_foreign");

            entity.HasIndex(e => e.Requirement, "idx_632056_events_positions_requirement_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('events_positions_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Callsign).HasColumnName("callsign");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.End).HasColumnName("end");
            entity.Property(e => e.EventId).HasColumnName("event_id");
            entity.Property(e => e.Remark).HasColumnName("remark");
            entity.Property(e => e.Requirement).HasColumnName("requirement");
            entity.Property(e => e.Start).HasColumnName("start");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
        });

        modelBuilder.Entity<EventsPositionsBooking>(entity =>
        {
            entity.HasKey(e => e.EventsPositionId).HasName("idx_632065_primary");

            entity.ToTable("events_positions_bookings", "vatprc_atc");

            entity.HasIndex(e => e.Controller, "idx_632065_events_positions_bookings_controller_foreign");

            entity.Property(e => e.EventsPositionId).HasColumnName("events_position_id");
            entity.Property(e => e.Controller).HasColumnName("controller");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");

            entity.HasOne(d => d.ControllerNavigation).WithMany(p => p.EventsPositionsBookings)
                .HasForeignKey(d => d.Controller)
                .HasConstraintName("events_positions_bookings_controller_foreign");
        });

        modelBuilder.Entity<FailedJob>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632073_primary");

            entity.ToTable("failed_jobs", "vatprc_atc");

            entity.HasIndex(e => e.Uuid, "idx_632073_failed_jobs_uuid_unique").IsUnique();

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('failed_jobs_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Connection).HasColumnName("connection");
            entity.Property(e => e.Exception).HasColumnName("exception");
            entity.Property(e => e.FailedAt)
                .HasDefaultValueSql("CURRENT_TIMESTAMP")
                .HasColumnName("failed_at");
            entity.Property(e => e.Payload).HasColumnName("payload");
            entity.Property(e => e.Queue).HasColumnName("queue");
            entity.Property(e => e.Uuid)
                .HasMaxLength(191)
                .HasColumnName("uuid");
        });

        modelBuilder.Entity<Job>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632088_primary");

            entity.ToTable("jobs", "vatprc_atc");

            entity.HasIndex(e => e.Queue, "idx_632088_jobs_queue_index");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('jobs_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Attempts).HasColumnName("attempts");
            entity.Property(e => e.AvailableAt).HasColumnName("available_at");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Payload).HasColumnName("payload");
            entity.Property(e => e.Queue)
                .HasMaxLength(191)
                .HasColumnName("queue");
            entity.Property(e => e.ReservedAt).HasColumnName("reserved_at");
        });

        modelBuilder.Entity<Migration>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632101_primary");

            entity.ToTable("migrations", "vatprc_atc");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('migrations_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Batch).HasColumnName("batch");
            entity.Property(e => e.Migration1)
                .HasMaxLength(255)
                .HasColumnName("migration");
        });

        modelBuilder.Entity<Permission>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632109_primary");

            entity.ToTable("permissions", "vatprc_atc");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('permissions_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Description).HasColumnName("description");
            entity.Property(e => e.Name).HasColumnName("name");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
        });

        modelBuilder.Entity<Role>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632118_primary");

            entity.ToTable("roles", "vatprc_atc");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('roles_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Name).HasColumnName("name");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
        });

        modelBuilder.Entity<RolesInheritance>(entity =>
        {
            entity.HasKey(e => new { e.ParentId, e.ChildrenId }).HasName("idx_632126_primary");

            entity.ToTable("roles_inheritances", "vatprc_atc");

            entity.HasIndex(e => e.ChildrenId, "idx_632126_roles_inheritances_children_id_foreign");

            entity.Property(e => e.ParentId).HasColumnName("parent_id");
            entity.Property(e => e.ChildrenId).HasColumnName("children_id");
        });

        modelBuilder.Entity<RolesPermission>(entity =>
        {
            entity.HasKey(e => new { e.RoleId, e.PermissionId }).HasName("idx_632133_primary");

            entity.ToTable("roles_permissions", "vatprc_atc");

            entity.HasIndex(e => e.PermissionId, "idx_632133_roles_permissions_permission_id_foreign");

            entity.Property(e => e.RoleId).HasColumnName("role_id");
            entity.Property(e => e.PermissionId).HasColumnName("permission_id");
        });

        modelBuilder.Entity<Schedule>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632141_primary");

            entity.ToTable("schedules", "vatprc_atc");

            entity.HasIndex(e => e.UserId, "idx_632141_schedules_user_id_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('schedules_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Callsign).HasColumnName("callsign");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Finish).HasColumnName("finish");
            entity.Property(e => e.Remark).HasColumnName("remark");
            entity.Property(e => e.Start).HasColumnName("start");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
            entity.Property(e => e.UserId).HasColumnName("user_id");

            entity.HasOne(d => d.User).WithMany(p => p.Schedules)
                .HasForeignKey(d => d.UserId)
                .HasConstraintName("schedules_user_id_foreign");
        });

        modelBuilder.Entity<Train>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632152_primary");

            entity.ToTable("trains", "vatprc_atc");

            entity.HasIndex(e => e.Instructor, "idx_632152_trains_instructor_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('trains_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Content).HasColumnName("content");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Instructor).HasColumnName("instructor");
            entity.Property(e => e.ScheduledAt).HasColumnName("scheduled_at");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");

            entity.HasOne(d => d.InstructorNavigation).WithMany(p => p.Trains)
                .HasForeignKey(d => d.Instructor)
                .OnDelete(DeleteBehavior.ClientSetNull)
                .HasConstraintName("trains_instructor_foreign");
        });

        modelBuilder.Entity<TrainComment>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632169_primary");

            entity.ToTable("train_comments", "vatprc_atc");

            entity.HasIndex(e => e.Author, "idx_632169_train_comments_author_foreign");

            entity.HasIndex(e => e.LastEditor, "idx_632169_train_comments_last_editor_foreign");

            entity.HasIndex(e => e.Train, "idx_632169_train_comments_train_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('train_comments_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.Author).HasColumnName("author");
            entity.Property(e => e.Content).HasColumnName("content");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.LastEditor).HasColumnName("last_editor");
            entity.Property(e => e.Train).HasColumnName("train");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");

            entity.HasOne(d => d.AuthorNavigation).WithMany(p => p.TrainCommentAuthorNavigations)
                .HasForeignKey(d => d.Author)
                .OnDelete(DeleteBehavior.ClientSetNull)
                .HasConstraintName("train_comments_author_foreign");

            entity.HasOne(d => d.LastEditorNavigation).WithMany(p => p.TrainCommentLastEditorNavigations)
                .HasForeignKey(d => d.LastEditor)
                .OnDelete(DeleteBehavior.ClientSetNull)
                .HasConstraintName("train_comments_last_editor_foreign");
        });

        modelBuilder.Entity<TrainRequest>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632181_primary");

            entity.ToTable("train_requests", "vatprc_atc");

            entity.HasIndex(e => e.Student, "idx_632181_train_requests_student_foreign");

            entity.HasIndex(e => e.TrainId, "idx_632181_train_requests_train_id_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('train_requests_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.End).HasColumnName("end");
            entity.Property(e => e.Processed).HasColumnName("processed");
            entity.Property(e => e.Remark).HasColumnName("remark");
            entity.Property(e => e.Start).HasColumnName("start");
            entity.Property(e => e.Student).HasColumnName("student");
            entity.Property(e => e.TrainId).HasColumnName("train_id");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");

            entity.HasOne(d => d.StudentNavigation).WithMany(p => p.TrainRequests)
                .HasForeignKey(d => d.Student)
                .OnDelete(DeleteBehavior.ClientSetNull)
                .HasConstraintName("train_requests_student_foreign");
        });

        modelBuilder.Entity<TrainRequestsPeriod>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632194_primary");

            entity.ToTable("train_requests_periods", "vatprc_atc");

            entity.HasIndex(e => e.TrainRequestId, "idx_632194_train_requests_periods_train_request_id_foreign");

            entity.Property(e => e.Id)
                .HasDefaultValueSql("nextval('train_requests_periods_id_seq'::regclass)")
                .HasColumnName("id");
            entity.Property(e => e.End).HasColumnName("end");
            entity.Property(e => e.Start).HasColumnName("start");
            entity.Property(e => e.TrainRequestId).HasColumnName("train_request_id");
        });

        modelBuilder.Entity<TrainsBooking>(entity =>
        {
            entity.HasKey(e => e.TrainId).HasName("idx_632161_primary");

            entity.ToTable("trains_bookings", "vatprc_atc");

            entity.HasIndex(e => e.ClosedBy, "idx_632161_trains_bookings_closed_by_foreign");

            entity.HasIndex(e => e.Student, "idx_632161_trains_bookings_student_foreign");

            entity.Property(e => e.TrainId).HasColumnName("train_id");
            entity.Property(e => e.ClosedBy).HasColumnName("closed_by");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Remark).HasColumnName("remark");
            entity.Property(e => e.Student).HasColumnName("student");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");

            entity.HasOne(d => d.ClosedByNavigation).WithMany(p => p.TrainsBookingClosedByNavigations)
                .HasForeignKey(d => d.ClosedBy)
                .HasConstraintName("trains_bookings_closed_by_foreign");

            entity.HasOne(d => d.StudentNavigation).WithMany(p => p.TrainsBookingStudentNavigations)
                .HasForeignKey(d => d.Student)
                .OnDelete(DeleteBehavior.ClientSetNull)
                .HasConstraintName("trains_bookings_student_foreign");
        });

        modelBuilder.Entity<User>(entity =>
        {
            entity.HasKey(e => e.Id).HasName("idx_632204_primary");

            entity.ToTable("users", "vatprc_atc");

            entity.Property(e => e.Id).HasColumnName("id");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Email).HasColumnName("email");
            entity.Property(e => e.FirstName).HasColumnName("first_name");
            entity.Property(e => e.LastName).HasColumnName("last_name");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");
            entity.Property(e => e.VatsimDivision).HasColumnName("vatsim_division");
            entity.Property(e => e.VatsimRating)
                .HasDefaultValueSql("'0'::bigint")
                .HasColumnName("vatsim_rating");

            entity.HasMany(d => d.Instructors).WithMany(p => p.Students)
                .UsingEntity<Dictionary<string, object>>(
                    "UsersInstructor",
                    r => r.HasOne<User>().WithMany()
                        .HasForeignKey("InstructorId")
                        .HasConstraintName("users_instructors_instructor_id_foreign"),
                    l => l.HasOne<User>().WithMany()
                        .HasForeignKey("StudentId")
                        .HasConstraintName("users_instructors_student_id_foreign"),
                    j =>
                    {
                        j.HasKey("InstructorId", "StudentId").HasName("idx_632216_primary");
                        j.ToTable("users_instructors", "vatprc_atc");
                        j.HasIndex(new[] { "StudentId" }, "idx_632216_users_instructors_student_id_foreign");
                        j.IndexerProperty<decimal>("InstructorId").HasColumnName("instructor_id");
                        j.IndexerProperty<decimal>("StudentId").HasColumnName("student_id");
                    });

            entity.HasMany(d => d.Students).WithMany(p => p.Instructors)
                .UsingEntity<Dictionary<string, object>>(
                    "UsersInstructor",
                    r => r.HasOne<User>().WithMany()
                        .HasForeignKey("StudentId")
                        .HasConstraintName("users_instructors_student_id_foreign"),
                    l => l.HasOne<User>().WithMany()
                        .HasForeignKey("InstructorId")
                        .HasConstraintName("users_instructors_instructor_id_foreign"),
                    j =>
                    {
                        j.HasKey("InstructorId", "StudentId").HasName("idx_632216_primary");
                        j.ToTable("users_instructors", "vatprc_atc");
                        j.HasIndex(new[] { "StudentId" }, "idx_632216_users_instructors_student_id_foreign");
                        j.IndexerProperty<decimal>("InstructorId").HasColumnName("instructor_id");
                        j.IndexerProperty<decimal>("StudentId").HasColumnName("student_id");
                    });
        });

        modelBuilder.Entity<UsersRole>(entity =>
        {
            entity.HasKey(e => new { e.UserId, e.RoleId }).HasName("idx_632223_primary");

            entity.ToTable("users_roles", "vatprc_atc");

            entity.HasIndex(e => e.RoleId, "idx_632223_users_roles_role_id_foreign");

            entity.Property(e => e.UserId).HasColumnName("user_id");
            entity.Property(e => e.RoleId).HasColumnName("role_id");
            entity.Property(e => e.ExpirationTime).HasColumnName("expiration_time");

            entity.HasOne(d => d.User).WithMany(p => p.UsersRoles)
                .HasForeignKey(d => d.UserId)
                .HasConstraintName("users_roles_user_id_foreign");
        });

        modelBuilder.Entity<UsersToken>(entity =>
        {
            entity.HasKey(e => e.UserId).HasName("idx_632230_primary");

            entity.ToTable("users_tokens", "vatprc_atc");

            entity.Property(e => e.UserId).HasColumnName("user_id");
            entity.Property(e => e.AccessToken).HasColumnName("access_token");
            entity.Property(e => e.ApplicationToken).HasColumnName("application_token");
            entity.Property(e => e.CreatedAt).HasColumnName("created_at");
            entity.Property(e => e.Expiration).HasColumnName("expiration");
            entity.Property(e => e.RefreshToken).HasColumnName("refresh_token");
            entity.Property(e => e.TokenExpires).HasColumnName("token_expires");
            entity.Property(e => e.UpdatedAt).HasColumnName("updated_at");

            entity.HasOne(d => d.User).WithOne(p => p.UsersToken)
                .HasForeignKey<UsersToken>(d => d.UserId)
                .HasConstraintName("users_tokens_user_id_foreign");
        });

        OnModelCreatingPartial(modelBuilder);
    }

    partial void OnModelCreatingPartial(ModelBuilder modelBuilder);
}
