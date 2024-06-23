using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/slots/{sid}/booking")]
public class EventSlotBookingController(VATPRCContext DbContext) : ControllerBase
{
    protected async Task<EventBooking> LoadAsync(Ulid eid, Ulid sid)
    {
        var slot = await DbContext.EventSlot
            .Include(x => x.EventAirspace)
                .ThenInclude(x => x.Event)
            .Include(x => x.Booking)
            .SingleOrDefaultAsync(x => x.Id == sid && x.EventAirspace.EventId == eid)
            ?? throw new ApiError.EventSlotNotFound(eid, sid);
        if (slot.Booking == null) throw new ApiError.EventSlotNotBooked(eid, sid);
        return slot.Booking;
    }

    public record EventBookingDto
    {
        public Ulid Id { get; set; }
        public Ulid UserId { get; set; }
        public DateTimeOffset CreatedAt { get; set; }
        public DateTimeOffset UpdatedAt { get; set; }

        public EventBookingDto(EventBooking booking)
        {
            Id = booking.Id;
            UserId = booking.UserId;
            CreatedAt = booking.CreatedAt;
            UpdatedAt = booking.UpdatedAt;
        }
    }

    [HttpGet]
    public async Task<EventBookingDto> Get(Ulid eid, Ulid sid)
    {
        var booking = await LoadAsync(eid, sid);
        return new(booking);
    }

    [HttpPut]
    [ApiError.Has<ApiError.EventNotFound>]
    public async Task<EventBookingDto> Put(Ulid eid, Ulid sid)
    {
        var uid = Ulid.Parse(User.Identity!.Name);
        var booking = await LoadAsync(eid, sid);
        if (booking != null) throw new ApiError.EventSlotBooked(eid, sid);
        var bookCount = await DbContext.EventBooking.CountAsync(x => x.UserId == uid && x.EventSlot.EventAirspace.EventId == eid);
        if (bookCount > 1)
        {
            throw new ApiError.EventBookMaximumExceeded(eid);
        }
        booking = new()
        {
            UserId = uid,
            EventSlotId = sid,
        };
        await DbContext.SaveChangesAsync();
        return new(booking);
    }

    [HttpDelete]
    public async Task<EventBookingDto> Delete(Ulid eid, Ulid sid)
    {
        var booking = await LoadAsync(eid, sid);
        if (booking.UserId.ToString() != User.Identity?.Name && !User.IsInRole(Models.User.UserRoles.Admin))
        {
            throw new ApiError.EventSlotBookedByAnotherUser(eid, sid);
        }
        DbContext.EventBooking.Remove(booking);
        await DbContext.SaveChangesAsync();
        return new(booking);
    }
}
