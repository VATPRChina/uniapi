using System.Collections.Immutable;
using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Services;

public class AtcPositionKindService
{
    protected readonly ImmutableDictionary<string, AtcPositionKind> positionKinds;

    public AtcPositionKindService()
    {
        positionKinds = new AtcPositionKind[]{
            new(){ Id = "DEL", NameEn = "Delivery", NameZh = "放行", IsTrainable = false,},
            new(){ Id = "GND", NameEn = "Ground", NameZh = "地面" },
            new() { Id = "TWR", NameEn = "Tower", NameZh = "塔台" },
            new() { Id = "T2", NameEn = "Tier 2", NameZh = "2 类席位" },
            new(){ Id = "APP", NameEn = "Approach", NameZh = "进近" },
            new(){ Id = "CTR", NameEn = "Center", NameZh = "区域" },
            new(){ Id = "FSS", NameEn = "FSS", NameZh = "飞服", IsTrainable = false },
            new(){ Id = "FMP", NameEn = "FMP", NameZh = "流量控制", IsTrainable = false },
        }.ToImmutableDictionary(x => x.Id);
    }

    public AtcPositionKind? GetById(string id)
        => positionKinds.GetValueOrDefault(id);

    public IEnumerable<AtcPositionKind> GetAll()
        => positionKinds.Values;
}
