using System;
using System.Collections.Generic;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class Migration
{
    public long Id { get; set; }

    public string Migration1 { get; set; } = null!;

    public long Batch { get; set; }
}
