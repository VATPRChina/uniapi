using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class ApplicationsInterview
{
    public long Id { get; set; }

    public decimal ApplicationId { get; set; }

    public decimal Interviewer { get; set; }

    public DateTime ScheduleTime { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual User InterviewerNavigation { get; set; } = null!;
}
