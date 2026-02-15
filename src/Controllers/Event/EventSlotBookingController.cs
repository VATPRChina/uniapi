using System.Collections.Concurrent;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Event;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/slots/{sid}/booking")]
public class EventSlotBookingController(
    DatabaseAdapter DbContext,
    IUserAccessor userAccessor) : ControllerBase
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
                .ThenInclude(b => b!.User)
            .SingleOrDefaultAsync(x => x.Id == sid && x.EventAirspace.EventId == eid)
            ?? throw new ApiError.EventSlotNotFound(eid, sid);
        return slot.Booking;
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<EventBookingDto> Get(Ulid eid, Ulid sid)
    {
        var booking = await LoadAsync(eid, sid) ?? throw new ApiError.EventSlotNotBooked(eid, sid);
        var includeUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
        return EventBookingDto.From(booking, includeUser);
    }

    [HttpPut]
    [ApiError.Has<ApiError.EventNotFound>]
    public async Task<EventBookingDto> Put(Ulid eid, Ulid sid, CancellationToken ct)
    {
        var lockObject = UserLevelLock.GetOrAdd(userAccessor.GetUserId(), new SemaphoreSlim(1, 1));
        await lockObject.WaitAsync(ct);
        try
        {
            var uid = userAccessor.GetUserId();
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
            var includeUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
            return EventBookingDto.From(booking, includeUser);
        }
        finally
        {
            lockObject.Release();
        }
    }

    [HttpDelete]
    public async Task<EventBookingDto> Delete(Ulid eid, Ulid sid)
    {
        var lockObject = UserLevelLock.GetOrAdd(userAccessor.GetUserId(), new SemaphoreSlim(1, 1));
        await lockObject.WaitAsync();
        try
        {
            var booking = await LoadAsync(eid, sid) ?? throw new ApiError.EventSlotNotBooked(eid, sid);
            if (!booking.EventSlot.EventAirspace.Event.IsInBookingPeriod)
            {
                throw new ApiError.EventNotInBookingTime(eid);
            }
            if (booking.UserId != userAccessor.GetUserId() && !User.IsInRole(UserRoles.Staff))
            {
                throw new ApiError.EventSlotBookedByAnotherUser(eid, sid);
            }
            DbContext.EventBooking.Remove(booking);
            await DbContext.SaveChangesAsync();
            var includeUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
            return EventBookingDto.From(booking, includeUser);
        }
        finally
        {
            lockObject.Release();
        }
    }
}
