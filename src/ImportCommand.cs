using System.Collections.Frozen;
using System.CommandLine;
using System.CommandLine.Invocation;
using Amazon.Runtime;
using Amazon.S3;
using Arinc424;
using Microsoft.Data.Sqlite;
using Net.Vatprc.AtcApi;
using Net.Vatprc.Uniapi.Controllers;
using Net.Vatprc.Uniapi.Controllers.Atc;
using Net.Vatprc.Uniapi.Models.Event;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;
using nietras.SeparatedValues;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi;

public class ImportCommand : Command
{
    protected readonly WebApplication App;

    public ImportCommand(WebApplication app) : base("import", "Import navdata")
    {
        App = app;
        this.SetHandler(Handle);
    }

    protected async Task Handle(InvocationContext context)
    {
        using var scope = App.Services.CreateScope();
        using var db = scope.ServiceProvider.GetRequiredService<Database>();
        var sheet = scope.ServiceProvider.GetRequiredService<SheetService>();

        using var atc = new AtcContext();

        // var events = await atc.Events.ToListAsync();
        // foreach (var evnt in events)
        // {
        //     var newEvent = new Models.Event.Event
        //     {
        //         Id = Ulid.NewUlid(evnt.CreatedAt ?? DateTimeOffset.MinValue),
        //         Title = evnt.Title,
        //         Description = "(imported)" + (evnt.Remark ?? string.Empty),
        //         StartAt = evnt.Start,
        //         EndAt = evnt.Finish,
        //         CreatedAt = (evnt.CreatedAt ?? DateTimeOffset.MinValue).ToUniversalTime(),
        //         UpdatedAt = (evnt.UpdatedAt ?? DateTimeOffset.MinValue).ToUniversalTime(),
        //         StartBookingAt = evnt.Start.ToUniversalTime(),
        //         EndBookingAt = evnt.Start.ToUniversalTime(),
        //         StartAtcBookingAt = evnt.OpenFrom?.ToUniversalTime(),
        //         ImageUrl = evnt.BannerUrl,
        //         CommunityLink = evnt.Url,
        //         VatsimLink = null,
        //     };
        //     db.Event.Add(newEvent);

        //     foreach (var evntAtc in await atc.EventsPositions.Where(p => p.EventId == evnt.Id).ToListAsync())
        //     {
        //         string positionKind = "DEL";
        //         if (evntAtc.Callsign.Contains("GND"))
        //         {
        //             positionKind = "GND";
        //         }
        //         else if (evntAtc.Callsign.Contains("TWR"))
        //         {
        //             positionKind = "TWR";
        //         }
        //         else if (evntAtc.Callsign.Contains("APP"))
        //         {
        //             positionKind = "APP";
        //         }
        //         else if (evntAtc.Callsign.Contains("CTR"))
        //         {
        //             positionKind = "CTR";
        //         }
        //         db.EventAtcPosition.Add(new EventAtcPosition
        //         {
        //             Id = Ulid.NewUlid(evnt.CreatedAt ?? evnt.Start),
        //             EventId = newEvent.Id,
        //             Callsign = evntAtc.Callsign,
        //             StartAt = (evntAtc.Start ?? evnt.Start).ToUniversalTime(),
        //             EndAt = (evntAtc.End ?? evnt.Finish).ToUniversalTime(),
        //             PositionKindId = positionKind,
        //             MinimumControllerState = Models.Atc.UserAtcPermission.UserControllerState.Solo,
        //             Remarks = "(imported)" + evntAtc.Remark,
        //         });
        //     }
        // }

        var existingUsers = await db.User.ToListAsync();
        // foreach (var user in await atc.Users.ToListAsync())
        // {
        //     if (existingUsers.Any(u => u.Cid == user.Id.ToString()))
        //     {
        //         continue;
        //     }
        //     if (existingUsers.Any(u => u.Email == user.Email))
        //     {
        //         continue;
        //     }
        //     var nuser = new Models.User
        //     {
        //         Id = Ulid.NewUlid(user.CreatedAt ?? DateTimeOffset.MinValue),
        //         Cid = user.Id.ToString(),
        //         FullName = $"{user.FirstName} {user.LastName}",
        //         Email = user.Email,
        //         CreatedAt = (user.CreatedAt ?? DateTimeOffset.MinValue).ToUniversalTime(),
        //         UpdatedAt = DateTimeOffset.UtcNow,
        //     };
        //     db.User.Add(nuser);
        // }

        // foreach (var user in await atc.Users.ToListAsync())
        // {
        //     var nuser = existingUsers.FirstOrDefault(u => u.Cid == user.Id.ToString());
        //     if (nuser == null)
        //     {
        //         continue;
        //     }
        //     var roles = await atc.UsersRoles.Where(r => r.UserId == user.Id).ToListAsync();
        //     // 300	DEL Full Permission
        //     if (roles.Any(r => r.RoleId == 300))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "DEL",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        //     // 301	DEL Solo Permission
        //     else if (roles.Any(r => r.RoleId == 301))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "DEL",
        //             State = UserControllerState.Solo,
        //             SoloExpiresAt = roles.First(r => r.RoleId == 311).ExpirationTime,
        //         });
        //     }
        //     // 302	DEL Under Mentoring Permission
        //     else if (roles.Any(r => r.RoleId == 302))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "DEL",
        //             State = UserControllerState.UnderMentor,
        //         });
        //     }
        //     // 310	GND Full Permission
        //     if (roles.Any(r => r.RoleId == 310))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "GND",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        //     // 311	GND Solo Permission
        //     else if (roles.Any(r => r.RoleId == 311))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "GND",
        //             State = UserControllerState.Solo,
        //             SoloExpiresAt = roles.First(r => r.RoleId == 311).ExpirationTime,
        //         });
        //     }
        //     // 312	GND Under Mentoring Permission
        //     else if (roles.Any(r => r.RoleId == 312))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "GND",
        //             State = UserControllerState.UnderMentor,
        //         });
        //     }
        //     // 320	TWR Full Permission
        //     if (roles.Any(r => r.RoleId == 320))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "TWR",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        //     // 321	TWR Solo Permission
        //     else if (roles.Any(r => r.RoleId == 321))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "TWR",
        //             State = UserControllerState.Solo,
        //             SoloExpiresAt = roles.First(r => r.RoleId == 321).ExpirationTime,
        //         });
        //     }
        //     // 322	TWR Under Mentoring Permission
        //     else if (roles.Any(r => r.RoleId == 322))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "TWR",
        //             State = UserControllerState.UnderMentor,
        //         });
        //     }
        //     // 330	APP Full Permission
        //     if (roles.Any(r => r.RoleId == 330))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "APP",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        //     // 331	APP Solo Permission
        //     else if (roles.Any(r => r.RoleId == 331))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "APP",
        //             State = UserControllerState.Solo,
        //             SoloExpiresAt = roles.First(r => r.RoleId == 331).ExpirationTime,
        //         });
        //     }
        //     // 332	APP Under Mentoring Permission
        //     else if (roles.Any(r => r.RoleId == 332))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "APP",
        //             State = UserControllerState.UnderMentor,
        //         });
        //     }
        //     // 333	Tier 2 Permission
        //     else if (roles.Any(r => r.RoleId == 333))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "T2",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        //     // 340	CTR Full Permission
        //     if (roles.Any(r => r.RoleId == 340))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "CTR",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        //     // 341	CTR Solo Permission
        //     else if (roles.Any(r => r.RoleId == 341))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "CTR",
        //             State = UserControllerState.Solo,
        //             SoloExpiresAt = roles.First(r => r.RoleId == 341).ExpirationTime,
        //         });
        //     }
        //     // 342	CTR Under Mentoring Permission
        //     else if (roles.Any(r => r.RoleId == 342))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "CTR",
        //             State = UserControllerState.UnderMentor,
        //         });
        //     }
        //     // 350	FSS Full Permission
        //     if (roles.Any(r => r.RoleId == 350))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "FSS",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        //     // 351	FSS Solo Permission
        //     else if (roles.Any(r => r.RoleId == 351))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "FSS",
        //             State = UserControllerState.Solo,
        //             SoloExpiresAt = roles.First(r => r.RoleId == 351).ExpirationTime,
        //         });
        //     }
        //     // 352	FSS Under Mentoring Permission
        //     else if (roles.Any(r => r.RoleId == 352))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "FSS",
        //             State = UserControllerState.UnderMentor,
        //         });
        //     }
        //     // 360	Traffic Management Center Permission
        //     if (roles.Any(r => r.RoleId == 360))
        //     {
        //         db.UserAtcPermission.Add(new Models.Atc.UserAtcPermission
        //         {
        //             UserId = nuser.Id,
        //             PositionKindId = "FMP",
        //             State = UserControllerState.Certified,
        //         });
        //     }
        // }

        // foreach (var app in await atc.Applications.ToListAsync())
        // {
        //     var appMeta = await atc.ApplicationsMetas.Where(m => m.ApplicationId == app.Id).FirstAsync();

        //     Models.Atc.AtcApplicationStatus status = Models.Atc.AtcApplicationStatus.Submitted;
        //     if (app.ProcessedAt != null)
        //     {
        //         if (app.Accepted)
        //         {
        //             status = Models.Atc.AtcApplicationStatus.Approved;
        //         }
        //         else
        //         {
        //             status = Models.Atc.AtcApplicationStatus.Rejected;
        //         }
        //     }

        //     var userId = existingUsers.FirstOrDefault(u => u.Cid == app.Applicant.ToString())?.Id ?? Ulid.Empty;

        //     if (userId == Ulid.Empty)
        //     {
        //         userId = Ulid.NewUlid(app.CreatedAt ?? DateTimeOffset.MinValue);
        //         var nuser = new Models.User
        //         {
        //             Id = userId,
        //             Cid = app.Applicant.ToString(),
        //             FullName = app.Applicant.ToString(),
        //             Email = null,
        //             CreatedAt = (app.CreatedAt ?? DateTimeOffset.MinValue).ToUniversalTime(),
        //             UpdatedAt = DateTimeOffset.UtcNow,
        //         };
        //         db.User.Add(nuser);
        //         existingUsers.Add(nuser);
        //     }

        //     var applySheet = await sheet.SetSheetFilingAsync(UserAtcApplicationController.ATC_APPLICATION_SHEET_ID, null, userId, new Dictionary<string, string>
        //     {
        //         { "age", appMeta.Age.ToString()},
        //         { "occupation", appMeta.Occupation },
        //         { "location", appMeta.Location},
        //         { "previous_experience", appMeta.PreviousAtc switch
        //         {
        //             true => "有（如果可能，请在下方的简介中描述）",
        //             false => "无",
        //         }},
        //         { "weekly_hours", appMeta.WeeklyHours.ToString() },
        //         { "english_level", appMeta.EnglishLevel switch
        //         {
        //             5 =>"英文交流基本无任何障碍",
        //             4 =>"除了熟练运用陆空对话外，还能用英文处理一些非常规情景的对话。",
        //             3 =>"能听懂大多数场景下的陆空对话，能用语音进行流利的英语对话",
        //             2 =>"勉强能听懂和进行陆空对话",
        //             1 =>"粗通英文，可以借助文字和翻译器进行英文陆空对话",
        //             0 =>"不懂英文或几乎不懂英文",
        //             _ => $"未知（{appMeta.EnglishLevel}）",
        //         }
        //         },
        //         { "self_introduction", appMeta.SelfIntroduction },
        //         { "expectation", appMeta.Expectation },
        //     });

        //     var reviewSheet = await sheet.SetSheetFilingAsync(AtcApplicationController.ATC_APPLICATION_REVIEW_SHEET_ID, null, userId, new Dictionary<string, string>
        //     {
        //         { "review", appMeta.Remark ?? string.Empty },
        //     });

        //     var napp = new Models.Atc.AtcApplication
        //     {
        //         Id = Ulid.NewUlid(app.CreatedAt ?? DateTimeOffset.MinValue),
        //         UserId = userId,
        //         AppliedAt = (app.CreatedAt ?? DateTimeOffset.MinValue).ToUniversalTime(),
        //         Status = status,
        //         ApplicationFilingId = applySheet.Id,
        //         ReviewFilingId = reviewSheet.Id,
        //     };
        //     db.AtcApplication.Add(napp);
        // }

        // foreach (var train in await atc.Trains.ToListAsync())
        // {
        //     var booking = await atc.TrainsBookings.Where(b => b.TrainId == train.Id).SingleOrDefaultAsync();
        //     if (booking == null)
        //     {
        //         Console.WriteLine($"跳过培训 {train.Id}，因为找不到对应的预订记录");
        //         continue;
        //     }

        //     var reviewSheet = await sheet.SetSheetFilingAsync(TrainingController.RECORD_SHEEET_ID, null, existingUsers.Single(u => u.Cid == (booking.ClosedBy ?? train.Instructor).ToString()).Id, new Dictionary<string, string>
        //     {
        //         { "review", booking.Remark ?? string.Empty },
        //     });
        //     var ntrain = new Models.Atc.Training
        //     {
        //         Id = Ulid.NewUlid(train.CreatedAt ?? DateTimeOffset.MinValue),
        //         Name = train.Content ?? "导入的培训",
        //         TrainerId = existingUsers.Single(u => u.Cid == train.Instructor.ToString()).Id,
        //         TraineeId = existingUsers.Single(u => u.Cid == booking.Student.ToString()).Id,
        //         StartAt = train.ScheduledAt,
        //         EndAt = train.ScheduledAt,
        //         CreatedAt = (train.CreatedAt ?? DateTimeOffset.MinValue).ToUniversalTime(),
        //         UpdatedAt = (train.UpdatedAt ?? DateTimeOffset.UtcNow).ToUniversalTime(),
        //         RecordSheetFilingId = reviewSheet.Id,
        //     };

        //     db.Training.Add(ntrain);
        // }

        var etrains = await db.Training.ToListAsync();

        foreach (var req in await atc.TrainRequests.ToListAsync())
        {
            var strain = await atc.Trains.Where(t => t.Id == req.TrainId).FirstOrDefaultAsync();
            var appId = Ulid.NewUlid(req.CreatedAt ?? DateTimeOffset.MinValue);
            Console.WriteLine($"导入培训申请 {appId}");
            var nreq = new Models.Atc.TrainingApplication
            {
                Id = appId,
                TraineeId = existingUsers.Single(u => u.Cid == req.Student.ToString()).Id,
                Name = req.Remark ?? "导入的培训申请",
                TrainId = etrains.SingleOrDefault(t => strain != null && t.CreatedAt == strain.CreatedAt && t.StartAt == strain.ScheduledAt)?.Id,
                CreatedAt = (req.CreatedAt ?? DateTimeOffset.MinValue).ToUniversalTime(),
                UpdatedAt = (req.UpdatedAt ?? DateTimeOffset.UtcNow).ToUniversalTime(),
            };
            db.TrainingApplication.Add(nreq);

            var eslots = await atc.TrainRequestsPeriods.Where(s => s.TrainRequestId == req.Id).ToListAsync();
            var slots = eslots.Select(s => new Models.Atc.TrainingApplicationSlot
            {
                Id = Ulid.NewUlid(req.CreatedAt ?? DateTimeOffset.MinValue),
                ApplicationId = appId,
                StartAt = s.Start.ToUniversalTime(),
                EndAt = s.End.ToUniversalTime(),
            }).ToList();
            db.TrainingApplicationSlot.AddRange(slots);
        }

        await db.SaveChangesAsync();
    }
}
