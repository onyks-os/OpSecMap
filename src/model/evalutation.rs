// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

use std::collections::HashSet;

use crate::{DependencyMap, ResourceId, RouteId, Scenario};

/// Why a resource is recorded as accessible in an evaluation.
#[derive(PartialEq, Eq)]
pub enum AccessOrigin {
    /// The scenario declares this resource available at the start.
    Initial,
    /// A route established access to this resource.
    Derived {
        /// The route that established access to the resource.
        route: RouteId,
    },
}

/// A resource together with the reason it became accessible.
#[derive(PartialEq, Eq)]
pub struct AccessEvidence {
    /// The accessible resource's stable identifier.
    resource: ResourceId,
    /// Whether access was initial or established through a route.
    origin: AccessOrigin,
}

impl AccessEvidence {
    /// Creates an evidence record for a resource and its access origin.
    #[must_use]
    pub fn new(resource: ResourceId, origin: AccessOrigin) -> Self {
        Self { resource, origin }
    }

    /// Returns the ID of the accessible resource.
    #[must_use]
    pub fn resource(&self) -> &ResourceId {
        &self.resource
    }

    /// Returns the reason this resource is accessible.
    #[must_use]
    pub fn origin(&self) -> &AccessOrigin {
        &self.origin
    }
}

/// The evidence collected for a single map and scenario evaluation.
pub struct Evaluation {
    /// One access record per accessible resource, once evaluation is complete.
    evidence: Vec<AccessEvidence>,
}

impl Evaluation {
    /// Validates the scenario's IDs and records initial and directly derived access.
    ///
    /// Evaluates routes for one round using only initially available resources.
    /// Further chains require additional rounds.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::UnknownInitialResource`] or
    /// [`EvaluationError::UnknownExcludedResource`] when an ID is absent from the map.
    pub fn evaluate(
        map: &DependencyMap,
        scenario: &Scenario,
    ) -> Result<Evaluation, EvaluationError> {
        for id in scenario.initially_available() {
            if map.find(id).is_none() {
                return Err(EvaluationError::UnknownInitialResource(id.clone()));
            }
        }
        for id in scenario.excluded() {
            if map.find(id).is_none() {
                return Err(EvaluationError::UnknownExcludedResource(id.clone()));
            }
        }

        let mut v = Vec::new();
        for i in scenario.initially_available() {
            let ae = AccessEvidence::new(i.clone(), AccessOrigin::Initial);
            v.push(ae);
        }

        let mut evaluation = Evaluation { evidence: v };
        let new_evidence = evaluation.expand_once(map, scenario);
        evaluation.evidence.extend(new_evidence);

        Ok(evaluation)
    }

    /// Returns the access evidence for `ri`, if the evaluation recorded any.
    #[must_use]
    pub fn evidence_for(&self, ri: &ResourceId) -> Option<&AccessEvidence> {
        self.evidence.iter().find(|i| i.resource() == ri)
    }

    /// Returns whether the evaluation recorded access to `ri`.
    #[must_use]
    pub fn is_accessible(&self, ri: &ResourceId) -> bool {
        self.evidence_for(ri).is_some()
    }

    /// Finds the resources reachable in one expansion round.
    ///
    /// Checks each route against access recorded before the round. Newly found
    /// resources cannot satisfy another route until a later round. Returns one
    /// `Derived` evidence per new target, choosing the first applicable route in
    /// map order. Already accessible and excluded targets are omitted.
    /// This method does not modify the evaluation.
    fn expand_once(&self, map: &DependencyMap, scenario: &Scenario) -> Vec<AccessEvidence> {
        let excluded = scenario.excluded();
        let mut new_evidence = Vec::new();
        let mut proposed = HashSet::new();
        for r in map.routes() {
            let target = r.target();
            if self.is_accessible(target) || excluded.contains(target) || proposed.contains(target)
            {
                continue;
            }
            if !r.prerequisites().iter().all(|id| self.is_accessible(id)) {
                continue;
            }
            new_evidence.push(AccessEvidence::new(
                target.clone(),
                AccessOrigin::Derived {
                    route: r.id().clone(),
                },
            ));
            proposed.insert(target);
        }
        new_evidence
    }
}

/// References to resources absent from the dependency map.
pub enum EvaluationError {
    /// An initially available resource does not exist in the map.
    UnknownInitialResource(ResourceId),
    /// An excluded resource does not exist in the map.
    UnknownExcludedResource(ResourceId),
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{AccessOrigin, Evaluation, EvaluationError};
    use crate::model::{DependencyMap, Resource, ResourceId, Route, RouteId, Scenario};

    fn id(value: &str) -> ResourceId {
        ResourceId::new(value.to_string()).expect("test ID must be valid")
    }

    fn map_with_resources() -> DependencyMap {
        let mut map = DependencyMap::new();
        for (resource_id, label) in [
            ("phone", "Phone"),
            ("password-store", "Password store"),
            ("email", "Email"),
        ] {
            let resource = Resource::new(id(resource_id), label.to_string())
                .expect("test resource must be valid");
            assert!(map.insert(resource).is_ok(), "test IDs must be distinct");
        }
        map
    }

    fn map_with_ids(ids: &[&str]) -> DependencyMap {
        let mut map = DependencyMap::new();
        for resource_id in ids {
            let resource = Resource::new(id(resource_id), (*resource_id).to_string())
                .expect("test resource must be valid");
            assert!(map.insert(resource).is_ok(), "test IDs must be distinct");
        }
        map
    }

    fn route_id(value: &str) -> RouteId {
        RouteId::new(value.to_string()).expect("test route ID must be valid")
    }

    fn add_route(map: &mut DependencyMap, name: &str, target: &str, prerequisites: &[&str]) {
        let route = Route::new(
            route_id(name),
            name.to_string(),
            id(target),
            prerequisites
                .iter()
                .map(|prerequisite| id(prerequisite))
                .collect(),
        )
        .expect("test route must be valid");
        assert!(map.insert_route(route).is_ok(), "test route must fit map");
    }

    fn scenario(initial: &[&str], excluded: &[&str]) -> Result<Scenario, &'static str> {
        Scenario::new(
            initial.iter().map(|value| id(value)).collect(),
            excluded.iter().map(|value| id(value)).collect(),
        )
        .map_err(|_| "test scenario was rejected")
    }

    #[test]
    fn initial_resources_have_initial_evidence_and_other_resources_are_inaccessible(
    ) -> Result<(), &'static str> {
        let map = map_with_resources();
        let scenario = Scenario::new(vec![id("phone"), id("password-store")], vec![])
            .map_err(|_| "test scenario was rejected")?;
        let evaluation = Evaluation::evaluate(&map, &scenario).map_err(|_| "evaluation failed")?;

        for initial in scenario.initially_available() {
            assert!(evaluation.is_accessible(initial));
            assert!(matches!(
                evaluation.evidence_for(initial),
                Some(evidence)
                    if evidence.resource() == initial
                        && matches!(evidence.origin(), AccessOrigin::Initial)
            ));
        }

        let email = id("email");
        assert!(map.find(&email).is_some());
        assert!(!evaluation.is_accessible(&email));
        assert!(evaluation.evidence_for(&email).is_none());
        Ok(())
    }

    #[test]
    fn unknown_initial_resource_reports_its_id() -> Result<(), &'static str> {
        let map = map_with_resources();
        let scenario = Scenario::new(vec![id("missing-initial")], vec![])
            .map_err(|_| "test scenario was rejected")?;

        assert!(matches!(
            Evaluation::evaluate(&map, &scenario),
            Err(EvaluationError::UnknownInitialResource(unknown))
                if unknown == id("missing-initial")
        ));
        Ok(())
    }

    #[test]
    fn unknown_excluded_resource_reports_its_id() -> Result<(), &'static str> {
        let map = map_with_resources();
        let scenario = Scenario::new(vec![id("phone")], vec![id("missing-excluded")])
            .map_err(|_| "test scenario was rejected")?;

        assert!(matches!(
            Evaluation::evaluate(&map, &scenario),
            Err(EvaluationError::UnknownExcludedResource(unknown))
                if unknown == id("missing-excluded")
        ));
        Ok(())
    }

    #[test]
    fn evaluation_leaves_map_and_scenario_available_for_reuse() -> Result<(), &'static str> {
        let map = map_with_resources();
        let scenario = Scenario::new(vec![id("phone")], vec![id("email")])
            .map_err(|_| "test scenario was rejected")?;

        Evaluation::evaluate(&map, &scenario).map_err(|_| "first evaluation failed")?;

        assert_eq!(map.find(&id("phone")).map(Resource::label), Some("Phone"));
        assert_eq!(map.resources().len(), 3);
        assert_eq!(scenario.initially_available(), &[id("phone")]);
        assert_eq!(scenario.excluded(), &[id("email")]);

        let repeated =
            Evaluation::evaluate(&map, &scenario).map_err(|_| "second evaluation failed")?;
        assert!(repeated.is_accessible(&id("phone")));
        assert!(matches!(
            repeated.evidence_for(&id("phone")),
            Some(evidence) if matches!(evidence.origin(), AccessOrigin::Initial)
        ));
        Ok(())
    }

    #[test]
    fn one_round_derives_target_when_all_prerequisites_are_accessible() -> Result<(), &'static str>
    {
        let mut map = map_with_ids(&["email", "password-store", "phone"]);
        add_route(
            &mut map,
            "email-login",
            "email",
            &["password-store", "phone"],
        );
        let scenario = scenario(&["password-store", "phone"], &[])?;

        let evaluation = Evaluation::evaluate(&map, &scenario).map_err(|_| "evaluation failed")?;

        assert!(evaluation.is_accessible(&id("email")));
        assert!(matches!(
            evaluation.evidence_for(&id("email")),
            Some(evidence)
                if evidence.resource() == &id("email")
                    && matches!(evidence.origin(), AccessOrigin::Derived { route }
                        if route == &route_id("email-login"))
        ));
        Ok(())
    }

    #[test]
    fn one_round_does_not_derive_target_with_missing_prerequisite() -> Result<(), &'static str> {
        let mut map = map_with_ids(&["email", "password-store", "phone"]);
        add_route(
            &mut map,
            "email-login",
            "email",
            &["password-store", "phone"],
        );
        let scenario = scenario(&["password-store"], &[])?;

        let evaluation = Evaluation::evaluate(&map, &scenario).map_err(|_| "evaluation failed")?;

        assert!(!evaluation.is_accessible(&id("email")));
        assert!(evaluation.evidence_for(&id("email")).is_none());
        Ok(())
    }

    #[test]
    fn one_round_can_use_alternative_route_when_login_is_blocked() -> Result<(), &'static str> {
        let mut map = map_with_ids(&["email", "password-store", "phone", "recovery-sheet"]);
        add_route(
            &mut map,
            "email-login",
            "email",
            &["password-store", "phone"],
        );
        add_route(&mut map, "email-recovery", "email", &["recovery-sheet"]);
        let scenario = scenario(&["recovery-sheet"], &["phone"])?;

        let evaluation = Evaluation::evaluate(&map, &scenario).map_err(|_| "evaluation failed")?;

        assert!(matches!(
            evaluation.evidence_for(&id("email")),
            Some(evidence)
                if matches!(evidence.origin(), AccessOrigin::Derived { route }
                    if route == &route_id("email-recovery"))
        ));
        Ok(())
    }

    #[test]
    fn one_round_does_not_derive_excluded_target() -> Result<(), &'static str> {
        let mut map = map_with_ids(&["email", "recovery-sheet"]);
        add_route(&mut map, "email-recovery", "email", &["recovery-sheet"]);
        let scenario = scenario(&["recovery-sheet"], &["email"])?;

        let evaluation = Evaluation::evaluate(&map, &scenario).map_err(|_| "evaluation failed")?;

        assert!(!evaluation.is_accessible(&id("email")));
        assert!(evaluation.evidence_for(&id("email")).is_none());
        Ok(())
    }

    #[test]
    fn one_round_uses_first_satisfied_route_without_duplicate_evidence() -> Result<(), &'static str>
    {
        let mut map = map_with_ids(&["email", "phone", "recovery-sheet"]);
        add_route(&mut map, "email-phone", "email", &["phone"]);
        add_route(&mut map, "email-recovery", "email", &["recovery-sheet"]);
        let scenario = scenario(&["phone", "recovery-sheet"], &[])?;

        let evaluation = Evaluation::evaluate(&map, &scenario).map_err(|_| "evaluation failed")?;

        assert_eq!(
            evaluation
                .evidence
                .iter()
                .filter(|evidence| evidence.resource() == &id("email"))
                .count(),
            1
        );
        assert!(matches!(
            evaluation.evidence_for(&id("email")),
            Some(evidence)
                if matches!(evidence.origin(), AccessOrigin::Derived { route }
                    if route == &route_id("email-phone"))
        ));
        Ok(())
    }

    #[test]
    fn one_round_does_not_use_new_target_as_prerequisite_regardless_of_route_order(
    ) -> Result<(), &'static str> {
        for order in [["b-from-a", "c-from-b"], ["c-from-b", "b-from-a"]] {
            let mut map = map_with_ids(&["a", "b", "c"]);
            for route in order {
                match route {
                    "b-from-a" => add_route(&mut map, route, "b", &["a"]),
                    "c-from-b" => add_route(&mut map, route, "c", &["b"]),
                    _ => return Err("unexpected test route"),
                }
            }
            let scenario = scenario(&["a"], &[])?;

            let evaluation =
                Evaluation::evaluate(&map, &scenario).map_err(|_| "evaluation failed")?;

            assert!(matches!(
                evaluation.evidence_for(&id("b")),
                Some(evidence)
                    if matches!(evidence.origin(), AccessOrigin::Derived { route }
                        if route == &route_id("b-from-a"))
            ));
            assert!(!evaluation.is_accessible(&id("c")));
            assert!(evaluation.evidence_for(&id("c")).is_none());
        }
        Ok(())
    }
}
