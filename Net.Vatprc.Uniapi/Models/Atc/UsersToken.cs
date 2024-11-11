using System;
using System.Collections.Generic;

namespace Net.Vatprc.Uniapi.Models.Atc;

public class UsersToken
{
    public decimal UserId { get; set; }

    public string ApplicationToken { get; set; } = null!;

    public DateTime Expiration { get; set; }

    public string? AccessToken { get; set; }

    public string? RefreshToken { get; set; }

    public decimal? TokenExpires { get; set; }

    public DateTime? CreatedAt { get; set; }

    public DateTime? UpdatedAt { get; set; }

    public virtual User User { get; set; } = null!;
}
