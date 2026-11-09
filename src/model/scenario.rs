// Copyright (c) 2026 onyks-it
// SPDX-License-Identifier: AGPL-3.0-or-later
//!
//! Domain model for OpSecMap.

use std::collections::HashSet;

use super::resource::ResourceId;

/// Availability assumptions for one evaluation of a dependency map.
#[derive(Debug)]
pub struct Scenario {
    /// Resources directly accessible before any route is evaluated.
    initially_available: Vec<ResourceId>,
    /// Resources that must remain inaccessible throughout the evaluation.
    excluded: Vec<ResourceId>,
}

// Validate the assumptions before storing them.
impl Scenario {
    /// Creates a scenario from initial availability and exclusions.
    ///
    /// # Errors
    ///
    /// Returns an error containing the offending ID when either list has a
    /// duplicate or an ID appears in both lists.
    pub fn new(
        initially_available: Vec<ResourceId>,
        excluded: Vec<ResourceId>,
    ) -> Result<Self, ScenarioError> {
        let mut seen_initial = HashSet::new();
        for id in &initially_available {
            if !seen_initial.insert(id) {
                return Err(ScenarioError::DuplicateInitial(id.clone()));
            }
        }

        let mut seen_excluded = HashSet::new();
        let mut first_overlap = None;
        for id in &excluded {
            if !seen_excluded.insert(id) {
                return Err(ScenarioError::DuplicateExcluded(id.clone()));
            }
            if first_overlap.is_none() && seen_initial.contains(&id) {
                first_overlap = Some(id);
            }
        }
        if let Some(id) = first_overlap {
            return Err(ScenarioError::InitialAndExcluded(id.clone()));
        }

        Ok(Self {
            initially_available,
            excluded,
        })
    }

    /// Returns initially available IDs in their declared order.
    #[must_use]
    pub fn initially_available(&self) -> &[ResourceId] {
        &self.initially_available
    }

    /// Returns excluded IDs in their declared order.
    #[must_use]
    pub fn excluded(&self) -> &[ResourceId] {
        &self.excluded
    }
}

/// Invalid assumptions found while constructing a scenario.
pub enum ScenarioError {
    /// An ID occurs more than once among initially available resources.
    DuplicateInitial(ResourceId),
    /// An ID occurs more than once among excluded resources.
    DuplicateExcluded(ResourceId),
    /// The same resource is declared both available and excluded.
    InitialAndExcluded(ResourceId),
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{ResourceId, Scenario, ScenarioError};

    fn id(value: &str) -> ResourceId {
        ResourceId::new(value.to_string()).expect("test ID must be valid")
    }

    #[test]
    fn empty_scenario_is_valid() -> Result<(), &'static str> {
        let scenario = Scenario::new(vec![], vec![]).map_err(|_| "empty scenario was rejected")?;

        assert_eq!(scenario.initially_available(), []);
        assert_eq!(scenario.excluded(), []);
        Ok(())
    }

    #[test]
    fn valid_scenario_preserves_input_order_and_exposes_read_only_slices(
    ) -> Result<(), &'static str> {
        let scenario = Scenario::new(
            vec![id("password-store"), id("recovery-sheet")],
            vec![id("phone"), id("backup-phone")],
        )
        .map_err(|_| "valid scenario was rejected")?;

        let initial: &[ResourceId] = scenario.initially_available();
        let excluded: &[ResourceId] = scenario.excluded();
        assert_eq!(initial, &[id("password-store"), id("recovery-sheet")]);
        assert_eq!(excluded, &[id("phone"), id("backup-phone")]);
        Ok(())
    }

    #[test]
    fn duplicate_initial_reports_its_id() {
        let result = Scenario::new(vec![id("phone"), id("phone")], vec![]);

        assert!(matches!(
            result,
            Err(ScenarioError::DuplicateInitial(duplicate)) if duplicate == id("phone")
        ));
    }

    #[test]
    fn duplicate_excluded_reports_its_id() {
        let result = Scenario::new(vec![], vec![id("phone"), id("phone")]);

        assert!(matches!(
            result,
            Err(ScenarioError::DuplicateExcluded(duplicate)) if duplicate == id("phone")
        ));
    }

    #[test]
    fn initial_and_excluded_conflict_reports_its_id() {
        let result = Scenario::new(vec![id("phone")], vec![id("phone")]);

        assert!(matches!(
            result,
            Err(ScenarioError::InitialAndExcluded(conflict)) if conflict == id("phone")
        ));
    }

    #[test]
    fn multiple_conflicts_report_first_excluded_id() {
        let result = Scenario::new(
            vec![id("phone"), id("password-store")],
            vec![id("password-store"), id("phone")],
        );

        assert!(matches!(
            result,
            Err(ScenarioError::InitialAndExcluded(conflict)) if conflict == id("password-store")
        ));
    }

    #[test]
    fn duplicate_excluded_takes_precedence_over_overlap() {
        let result = Scenario::new(
            vec![id("phone")],
            vec![id("phone"), id("backup"), id("backup")],
        );

        assert!(matches!(
            result,
            Err(ScenarioError::DuplicateExcluded(duplicate)) if duplicate == id("backup")
        ));
    }
}
