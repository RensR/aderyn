use std::collections::BTreeMap;
use std::error::Error;

use crate::ast::{NodeID, NodeType};

use crate::capture;
use crate::context::browser::GetClosestAncestorOfTypeX;
use crate::detect::detector::IssueDetectorNamePool;
use crate::{
    context::workspace_context::WorkspaceContext,
    detect::detector::{IssueDetector, IssueSeverity},
};
use eyre::Result;

#[derive(Default)]
pub struct RevertsAndRequiresInLoopsDetector {
    // Keys are source file name and line number
    found_instances: BTreeMap<(String, usize, String), NodeID>,
}

impl IssueDetector for RevertsAndRequiresInLoopsDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        // Collect all require statements
        let requires_and_reverts = context
            .identifiers()
            .into_iter()
            .filter(|&id| id.name == "revert" || id.name == "require")
            .collect::<Vec<_>>();

        for item in requires_and_reverts {
            if let Some(_) = item.closest_ancestor_of_type(context, NodeType::ForStatement) {
                capture!(self, context, item);
            }
            if let Some(_) = item.closest_ancestor_of_type(context, NodeType::WhileStatement) {
                capture!(self, context, item);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        String::from("Loop contains `require`/`revert` statements")
    }

    fn description(&self) -> String {
        String::from("Avoid `require` / `revert` statements in a loop because a single bad item can cause the whole transaction to fail. It's better to forgive on fail and return failed elements post processing of the loop")
    }

    fn instances(&self) -> BTreeMap<(String, usize, String), NodeID> {
        self.found_instances.clone()
    }

    fn name(&self) -> String {
        format!("{}", IssueDetectorNamePool::RevertsAndRequiresInLoops)
    }
}

#[cfg(test)]
mod reevrts_and_requires_in_loops {
    use crate::detect::{
        detector::{detector_test_helpers::load_contract, IssueDetector},
        low::reverts_and_requries_in_loops::RevertsAndRequiresInLoopsDetector,
    };

    #[test]
    fn test_reverts_and_requires_in_loops() {
        let context = load_contract(
            "../tests/contract-playground/out/RevertsAndRequriesInLoops.sol/PotentialPanicInLoop.json",
        );

        let mut detector = RevertsAndRequiresInLoopsDetector::default();
        let found = detector.detect(&context).unwrap();

        // println!("{:?}", detector.instances());

        // assert that the detector found an issue
        assert!(found);
        // assert that the detector found the correct number of instances
        assert_eq!(detector.instances().len(), 2);
        // assert the severity is low
        assert_eq!(
            detector.severity(),
            crate::detect::detector::IssueSeverity::Low
        );
        // assert the title is correct
        assert_eq!(
            detector.title(),
            String::from("Loop contains `require`/`revert` statements")
        );
        // assert the description is correct
        assert_eq!(
            detector.description(),
            String::from("Avoid `require` / `revert` statements in a loop because a single bad item can cause the whole transaction to fail. It's better to forgive on fail and return failed elements post processing of the loop")
        );
    }
}
