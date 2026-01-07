using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class TrainsBooking
{
    public decimal TrainId { get; set; }

    public decimal Student { get; set; }

    public decimal? ClosedBy { get; set; }

    public string? Remark { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual User? ClosedByNavigation { get; set; }

    public virtual User StudentNavigation { get; set; } = null!;
}
