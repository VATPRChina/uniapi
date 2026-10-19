use itertools::Itertools;

use crate::modules::flight::flight_plan::parse_route;
use crate::modules::flight::flight_plan::validator::ValidatorError;
use crate::modules::navdata::models::{LegKind, PreferredRoute, ResolvedLeg};
use crate::modules::navdata::service::NavdataService;

pub async fn find_matching_route<'a>(
    navdata: &NavdataService,
    legs: &[ResolvedLeg],
    preferred_routes: &[&'a PreferredRoute],
) -> Result<Option<&'a PreferredRoute>, ValidatorError> {
    todo!()
}
