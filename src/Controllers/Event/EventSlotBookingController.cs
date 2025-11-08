using System.Collections.Concurrent;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/slots/{sid}/booking")]
public class EventSlotBookingController(Database DbContext) : ControllerBase
{
    /// <summary>
    /// User-level lock to ensure the booking request for a user is processed sequentially.
    /// This is not distributed lock, so it only works in a single instance.
    /// </summary>
    protected static ConcurrentDictionary<Ulid, SemaphoreSlim> UserLevelLock = new();

    protected async Task<EventBooking?> LoadAsync(Ulid eid, Ulid sid)
    {
        var slot = await DbContext.EventSlot
            .Include(x => x.EventAirspace)
                .ThenInclude(x => x.Event)
            .Include(x => x.Booking)
            .SingleOrDefaultAsync(x => x.Id == sid && x.EventAirspace.EventId == eid)
            ?? throw new ApiError.EventSlotNotFound(eid, sid);
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
    [AllowAnonymous]
    public async Task<EventBookingDto> Get(Ulid eid, Ulid sid)
    {
        var booking = await LoadAsync(eid, sid) ?? throw new ApiError.EventSlotNotBooked(eid, sid);
        return new(booking);
    }

    [HttpPut]
    [ApiError.Has<ApiError.EventNotFound>]
    public async Task<EventBookingDto> Put(Ulid eid, Ulid sid)
    {
        var lockObject = UserLevelLock.GetOrAdd(this.GetUserId(), new SemaphoreSlim(1, 1));
        await lockObject.WaitAsync();
        try
        {
            var uid = this.GetUserId();
            var booking = await LoadAsync(eid, sid);
            if (booking != null) throw new ApiError.EventSlotBooked(eid, sid);
            var eventt = await DbContext.Event.FindAsync(eid) ??
                throw new ApiError.EventNotFound(eid);
            if (!eventt.IsInBookingPeriod)
            {
                throw new ApiError.EventNotInBookingTime(eid);
            }
            // TODO: check if time overlaps
            booking = new()
            {
                UserId = uid,
                EventSlotId = sid,
            };
            DbContext.EventBooking.Add(booking);
            await DbContext.SaveChangesAsync();
            return new(booking);
        }
        finally
        {
            lockObject.Release();
        }
    }

    [HttpDelete]
    public async Task<EventBookingDto> Delete(Ulid eid, Ulid sid)
    {
        var lockObject = UserLevelLock.GetOrAdd(this.GetUserId(), new SemaphoreSlim(1, 1));
        await lockObject.WaitAsync();
        try
        {
            var booking = await LoadAsync(eid, sid) ?? throw new ApiError.EventSlotNotBooked(eid, sid);
            if (!booking.EventSlot.EventAirspace.Event.IsInBookingPeriod)
            {
                throw new ApiError.EventNotInBookingTime(eid);
            }
            if (booking.UserId != this.GetUserId() && !User.IsInRole(UserRoles.Staff))
            {
                throw new ApiError.EventSlotBookedByAnotherUser(eid, sid);
            }
            DbContext.EventBooking.Remove(booking);
            await DbContext.SaveChangesAsync();
            return new(booking);
        }
        finally
        {
            lockObject.Release();
        }
    }
}
