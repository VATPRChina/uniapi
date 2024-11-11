using System;
using System.Collections.Generic;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class Train
{
    public long Id { get; set; }

    public decimal Instructor { get; set; }

    public DateTime ScheduledAt { get; set; }

    public string? Content { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual User InstructorNavigation { get; set; } = null!;
}
