namespace Net.Vatprc.Uniapi.Dto;

public class TrainingApplicationCreateRequestSlot
{
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }
}
