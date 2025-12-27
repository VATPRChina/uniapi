using System.ComponentModel;

namespace Net.Vatprc.Uniapi.Dto;

public enum WarningMessageCode
{
  [Description("The aircraft does not support RVSM.")]
  no_rvsm,

  [Description("The aircraft does not support RNAV1.")]
  no_rnav1,

  [Description("The aircraft supports RNP AR with RF.")]
  rnp_ar,

  [Description("The aircraft supports RNP AR without RF.")]
  rnp_ar_without_rf,

  [Description("The aircraft does not have a transponder.")]
  no_transponder,

  [Description("The route contains a direct segment.")]
  route_direct_segment,

  [Description("The route contains a leg with an incorrect direction.")]
  route_leg_direction,

  [Description("The route contains a leg requiring controller approval.")]
  airway_require_approval,

  [Description("The route is not recommended.")]
  not_preferred_route,

  [Description("The cruising level type does not match the preferred route.")]
  cruising_level_mismatch,

  [Description("The cruising level is too low for the preferred route.")]
  cruising_level_too_low,

  [Description("The cruising level is not allowed for the preferred route.")]
  cruising_level_not_allowed,

  [Description("The planned route is matching a preferred route.")]
  route_match_preferred,
}
