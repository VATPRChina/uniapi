using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class TrainRequestsPeriod
{
    public long Id { get; set; }

    public decimal TrainRequestId { get; set; }

    public DateTime Start { get; set; }

    public DateTime End { get; set; }
}
