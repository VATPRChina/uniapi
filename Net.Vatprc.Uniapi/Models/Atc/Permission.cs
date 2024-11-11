using System;
using System.Collections.Generic;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class Permission
{
    public long Id { get; set; }

    public string Name { get; set; } = null!;

    public string? Description { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }
}
