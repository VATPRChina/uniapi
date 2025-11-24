using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/atc/bookings")]
public class AtcBookingController(
    Database DbContext,
    IUserAccessor userAccessor) : Controller
{
    [HttpGet]
    public async Task<IEnumerable<AtcBookingDto>> GetAtcBookings()
    {
        var user = await userAccessor.GetUser();

        return await DbContext.AtcBooking
            .OrderByDescending(b => b.StartAt)
            .Select(b => AtcBookingDto.From(b))
            .ToListAsync();
    }

    [HttpGet("mine")]
    public async Task<IEnumerable<AtcBookingDto>> GetMyAtcBookings()
    {
        var user = await userAccessor.GetUser();

        return await DbContext.AtcBooking
            .Where(b => b.UserId == user.Id)
            .OrderByDescending(b => b.StartAt)
            .Select(b => AtcBookingDto.From(b))
            .ToListAsync();
    }

    [HttpPost]
    [Authorize(Roles = UserRoles.Controller)]
    [ApiError.Has<ApiError.StartMustBeBeforeEnd>]
    public async Task<AtcBookingDto> CreateAtcBooking(AtcBookingSaveRequest req)
    {
        var user = await userAccessor.GetUser();

        if (req.StartTime >= req.EndTime)
        {
            throw new ApiError.StartMustBeBeforeEnd(nameof(req.StartTime), nameof(req.EndTime));
        }

        var booking = new Models.Atc.AtcBooking
        {
            Id = Ulid.NewUlid(),
            UserId = user.Id,
            Callsign = req.Callsign,
            BookedAt = DateTimeOffset.UtcNow,
            StartAt = req.StartTime.ToUniversalTime(),
            EndAt = req.EndTime.ToUniversalTime(),
        };
        DbContext.AtcBooking.Add(booking);
        await DbContext.SaveChangesAsync();
        return AtcBookingDto.From(booking);
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.AtcBookingNotFound>]
    public async Task<AtcBookingDto> GetAtcBooking(Ulid id)
    {
        var booking = await DbContext.AtcBooking.FindAsync(id) ??
            throw new ApiError.AtcBookingNotFound(id);

        return AtcBookingDto.From(booking);
    }

    [HttpPut("{id}")]
    [Authorize(Roles = UserRoles.Controller)]
    [ApiError.Has<ApiError.AtcBookingNotFound>]
    [ApiError.Has<ApiError.StartMustBeBeforeEnd>]
    [ApiError.Has<ApiError.AtcBookingForbidden>]
    public async Task<AtcBookingDto> UpdateAtcBooking(Ulid id, AtcBookingSaveRequest req)
    {
        var userId = userAccessor.GetUserId();
        var booking = await DbContext.AtcBooking.FindAsync(id) ??
            throw new ApiError.AtcBookingNotFound(id);

        if (booking.UserId != userId)
        {
            throw new ApiError.AtcBookingForbidden(id, booking.UserId);
        }

        if (req.StartTime >= req.EndTime)
        {
            throw new ApiError.StartMustBeBeforeEnd(nameof(req.StartTime), nameof(req.EndTime));
        }

        booking.Callsign = req.Callsign;
        booking.StartAt = req.StartTime.ToUniversalTime();
        booking.EndAt = req.EndTime.ToUniversalTime();
        booking.BookedAt = DateTimeOffset.UtcNow;
        await DbContext.SaveChangesAsync();
        return AtcBookingDto.From(booking);
    }

    [HttpDelete("{id}")]
    [Authorize(Roles = UserRoles.Controller)]
    [ApiError.Has<ApiError.AtcBookingNotFound>]
    [ApiError.Has<ApiError.AtcBookingForbidden>]
    public async Task<AtcBookingDto> DeleteAtcBooking(Ulid id)
    {
        var userId = userAccessor.GetUserId();
        var booking = await DbContext.AtcBooking.FindAsync(id) ??
            throw new ApiError.AtcBookingNotFound(id);
        if (booking.UserId != userId)
        {
            throw new ApiError.AtcBookingForbidden(id, booking.UserId);
        }

        DbContext.AtcBooking.Remove(booking);
        await DbContext.SaveChangesAsync();
        return AtcBookingDto.From(booking);
    }
}
