using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class EventsPosition
{
    public long Id { get; set; }

    public decimal EventId { get; set; }

    public string Callsign { get; set; } = null!;

    public decimal? Requirement { get; set; }

    public string? Remark { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public DateTime? Start { get; set; }

    public DateTime? End { get; set; }
}
