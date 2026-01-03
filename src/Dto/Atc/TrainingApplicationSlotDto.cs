namespace Net.Vatprc.Uniapi.Dto;

public class TrainingApplicationSlotDto
{
    public required Ulid Id { get; init; }
    public required Ulid ApplicationId { get; init; }
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }

    public static TrainingApplicationSlotDto From(Models.Atc.TrainingApplicationSlot slot)
    {
        return new TrainingApplicationSlotDto
        {
            Id = slot.Id,
            ApplicationId = slot.ApplicationId,
            StartAt = slot.StartAt,
            EndAt = slot.EndAt
        };
    }
}
