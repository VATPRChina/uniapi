using System;
using System.Collections.Generic;

namespace Net.Vatprc.AtcApi;

public partial class UsersRole
{
    public decimal UserId { get; set; }

    public decimal RoleId { get; set; }

    public DateTime? ExpirationTime { get; set; }

    public virtual User User { get; set; } = null!;
}
