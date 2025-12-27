using System.ComponentModel;

namespace Net.Vatprc.Uniapi.Dto;

public enum WarningMessageField
{
  [Description("Equipment")]
  equipment,

  [Description("Transponder")]
  transponder,

  [Description("Navigation Performance")]
  navigation_performance,

  [Description("Route, with field index if applicable")]
  route,

  [Description("Cruising Level")]
  cruising_level,
}
