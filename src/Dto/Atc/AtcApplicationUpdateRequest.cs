using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationUpdateRequest
{
    public required AtcApplicationStatus Status { get; set; }
}
