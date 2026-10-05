use super::*;
use crate::modules::navdata::models::{Airway, AnyFix, DirectionRestriction, GeoPoint, Sid, Star};

fn point(longitude: f64) -> AnyFix {
    AnyFix::GeoPoint(GeoPoint::new(30., longitude))
}

fn segment(from: f64, to: f64, restriction: DirectionRestriction) -> ResolvedLeg {
    ResolvedLeg {
        from: point(from),
        to: point(to),
        identifier: Some("A1".to_owned()),
        is_unknown: false,
        is_sid: false,
        is_star: false,
        direction_restriction: restriction,
    }
}

fn airway(legs: Vec<ResolvedLeg>) -> NavProc {
    NavProc::Airway(Airway {
        identifier: "A1".try_into().unwrap(),
        legs,
    })
}

fn expander(from: f64, to: f64, procedure: &NavProc) -> Expander<'_> {
    Expander::new(vec![ConstructedLeg {
        leg: segment(from, to, DirectionRestriction::None),
        procedure: Some(procedure),
    }])
}

#[test]
fn airway_expands_in_both_directions_without_changing_loaded_legs() {
    let published = vec![
        segment(110., 111., DirectionRestriction::Forward),
        segment(111., 112., DirectionRestriction::Backward),
        segment(112., 113., DirectionRestriction::None),
    ];
    let procedure = airway(published.clone());
    let forward = expander(110., 113., &procedure);
    assert_eq!(forward.expand().unwrap(), published);
    let expected: Vec<_> = published
        .iter()
        .rev()
        .cloned()
        .map(ResolvedLeg::into_reversed)
        .collect();
    assert_eq!(expander(113., 110., &procedure).expand().unwrap(), expected);
    assert_eq!(expander(113., 110., &procedure).expand().unwrap(), expected);
    assert_eq!(procedure.legs(), published);
}

#[test]
fn reverse_traversal_expands_every_segment_in_a_six_segment_airway() {
    let published: Vec<_> = (0..6)
        .map(|i| {
            segment(
                110. + f64::from(i),
                111. + f64::from(i),
                DirectionRestriction::None,
            )
        })
        .collect();
    let expanded = expander(116., 110., &airway(published)).expand().unwrap();
    assert_eq!(expanded.len(), 6);
    assert_eq!(expanded.first().unwrap().from, point(116.));
    assert_eq!(expanded.last().unwrap().to, point(110.));
    assert!(expanded.windows(2).all(|pair| pair[0].to == pair[1].from));
}

#[test]
fn sid_and_star_common_legs_are_not_reversed() {
    let published = vec![
        segment(110., 111., DirectionRestriction::None),
        segment(111., 112., DirectionRestriction::None),
    ];
    let procedures = [
        NavProc::Sid(Sid {
            airport: "AAAA".try_into().unwrap(),
            identifier: "A1".try_into().unwrap(),
            legs: published.clone(),
        }),
        NavProc::Star(Star {
            airport: "AAAA".try_into().unwrap(),
            identifier: "A1".try_into().unwrap(),
            legs: published.clone(),
        }),
    ];
    for procedure in procedures {
        assert_eq!(
            expander(110., 112., &procedure).expand().unwrap(),
            published
        );
        let reverse = expander(112., 110., &procedure);
        let original = reverse.route[0].leg.clone();
        assert_eq!(reverse.expand().unwrap(), vec![original]);
    }
}

#[test]
fn reverse_edges_do_not_connect_disjoint_sections_or_loop_forever() {
    let published = vec![
        segment(110., 111., DirectionRestriction::None),
        segment(111., 112., DirectionRestriction::None),
        segment(112., 110., DirectionRestriction::None),
        segment(113., 114., DirectionRestriction::None),
    ];
    let procedure = airway(published);
    let route = expander(114., 110., &procedure);
    let original = route.route[0].leg.clone();
    assert_eq!(route.expand().unwrap(), vec![original]);
}

#[test]
fn equal_length_paths_keep_published_record_order() {
    let published = vec![
        segment(110., 111., DirectionRestriction::None),
        segment(110., 112., DirectionRestriction::None),
        segment(111., 113., DirectionRestriction::None),
        segment(112., 113., DirectionRestriction::None),
    ];
    let expected = vec![published[0].clone(), published[2].clone()];
    let procedure = airway(published);
    assert_eq!(expander(110., 113., &procedure).expand().unwrap(), expected);
    let reverse: Vec<_> = expected
        .into_iter()
        .rev()
        .map(ResolvedLeg::into_reversed)
        .collect();
    assert_eq!(expander(113., 110., &procedure).expand().unwrap(), reverse);
}
