using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class User
{
    public decimal Id { get; set; }

    public string FirstName { get; set; } = null!;

    public string LastName { get; set; } = null!;

    public string Email { get; set; } = null!;

    public long VatsimRating { get; set; }

    public string VatsimDivision { get; set; } = null!;

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual ICollection<ApplicationsInterview> ApplicationsInterviews { get; set; } = new List<ApplicationsInterview>();

    public virtual ICollection<EventsPositionsBooking> EventsPositionsBookings { get; set; } = new List<EventsPositionsBooking>();

    public virtual ICollection<Schedule> Schedules { get; set; } = new List<Schedule>();

    public virtual ICollection<TrainComment> TrainCommentAuthorNavigations { get; set; } = new List<TrainComment>();

    public virtual ICollection<TrainComment> TrainCommentLastEditorNavigations { get; set; } = new List<TrainComment>();

    public virtual ICollection<TrainRequest> TrainRequests { get; set; } = new List<TrainRequest>();

    public virtual ICollection<Train> Trains { get; set; } = new List<Train>();

    public virtual ICollection<TrainsBooking> TrainsBookingClosedByNavigations { get; set; } = new List<TrainsBooking>();

    public virtual ICollection<TrainsBooking> TrainsBookingStudentNavigations { get; set; } = new List<TrainsBooking>();

    public virtual ICollection<UsersRole> UsersRoles { get; set; } = new List<UsersRole>();

    public virtual UsersToken? UsersToken { get; set; }

    public virtual ICollection<User> Instructors { get; set; } = new List<User>();

    public virtual ICollection<User> Students { get; set; } = new List<User>();
}
