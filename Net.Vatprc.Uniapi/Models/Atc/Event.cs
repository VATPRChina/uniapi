using System;
using System.Collections.Generic;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class Event
{
    public long Id { get; set; }

    public string Title { get; set; } = null!;

    public DateTime Start { get; set; }

    public DateTime Finish { get; set; }

    public string? Url { get; set; }

    public string? BannerUrl { get; set; }

    public string? Remark { get; set; }

    public bool Published { get; set; }

    public bool Open { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public DateTime? PublishFrom { get; set; }

    public DateTime? OpenFrom { get; set; }
}
