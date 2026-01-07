using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class TrainRequest
{
    public long Id { get; set; }

    public decimal Student { get; set; }

    public DateTime Start { get; set; }

    public DateTime End { get; set; }

    public decimal? TrainId { get; set; }

    public string? Remark { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public bool Processed { get; set; }

    public virtual User StudentNavigation { get; set; } = null!;
}
