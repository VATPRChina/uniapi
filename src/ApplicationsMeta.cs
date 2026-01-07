using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class ApplicationsMeta
{
    public decimal ApplicationId { get; set; }

    public long Age { get; set; }

    public string Occupation { get; set; } = null!;

    public string Location { get; set; } = null!;

    public bool PreviousAtc { get; set; }

    public long WeeklyHours { get; set; }

    public long EnglishLevel { get; set; }

    public string SelfIntroduction { get; set; } = null!;

    public string Expectation { get; set; } = null!;

    public string? Remark { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }
}
