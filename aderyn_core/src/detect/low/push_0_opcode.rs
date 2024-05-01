use std::{collections::BTreeMap, error::Error};

use crate::{
    ast::NodeID,
    capture,
    context::workspace_context::WorkspaceContext,
    detect::detector::{IssueDetector, IssueDetectorNamePool, IssueSeverity},
};
use eyre::Result;
use semver::{Op, VersionReq};

#[derive(Default)]
pub struct PushZeroOpcodeDetector {
    // Keys are: [0] source file name, [1] line number, [2] character location of node.
    // Do not add items manually, use `capture!` to add nodes to this BTreeMap.
    found_instances: BTreeMap<(String, usize, String), NodeID>,
}

fn version_req_allows_above_0_8_19(version_req: &VersionReq) -> bool {
    // Simplified logic to check if version_req allows versions above 0.8.19
    // Note: This is a basic example and might not cover all complex semver cases.
    if version_req.comparators.len() == 1 {
        let comparator = &version_req.comparators[0];
        match comparator.op {
            Op::Tilde | Op::Caret => {
                if comparator.major > 0 || comparator.minor >= Some(8) {
                    return true;
                }
            }
            Op::Greater | Op::GreaterEq => {
                return true;
            }
            Op::Exact => {
                if comparator.major == 0
                    && comparator.minor == Some(8)
                    && comparator.patch == Some(20)
                {
                    return true;
                }
            }
            _ => {}
        }
    } else if version_req.comparators.len() == 2 {
        let comparator_2 = &version_req.comparators[1];
        if comparator_2.major > 0
            || (comparator_2.minor >= Some(8))
            || (comparator_2.minor == Some(8) && comparator_2.patch >= Some(20))
        {
            return true;
        }
    }

    false
}

impl IssueDetector for PushZeroOpcodeDetector {
    fn detect(&mut self, context: &WorkspaceContext) -> Result<bool, Box<dyn Error>> {
        for pragma_directive in context.pragma_directives() {
            let mut version_string = String::new();

            for literal in &pragma_directive.literals {
                if literal == "solidity" {
                    continue;
                }
                if version_string.is_empty() && literal.contains("0.") {
                    version_string.push('=');
                }
                if version_string.len() > 5 && (literal == "<" || literal == "=") {
                    version_string.push(',');
                }
                version_string.push_str(literal);
            }
            let req = VersionReq::parse(&version_string)?;
            if version_req_allows_above_0_8_19(&req) {
                capture!(self, context, pragma_directive);
            }
        }

        Ok(!self.found_instances.is_empty())
    }

    fn severity(&self) -> IssueSeverity {
        IssueSeverity::Low
    }

    fn title(&self) -> String {
        String::from("PUSH0 is not supported by all chains")
    }

    fn description(&self) -> String {
        String::from("Solc compiler version 0.8.20 switches the default target EVM version to Shanghai, which means that the generated bytecode will include PUSH0 opcodes. Be sure to select the appropriate EVM version in case you intend to deploy on a chain other than mainnet like L2 chains that may not support PUSH0, otherwise deployment of your contracts will fail.")
    }

    fn instances(&self) -> BTreeMap<(String, usize, String), NodeID> {
        self.found_instances.clone()
    }

    fn name(&self) -> String {
        format!("{}", IssueDetectorNamePool::PushZeroOpcode)
    }
}

#[cfg(test)]
mod unspecific_solidity_pragma_tests {
    use crate::detect::detector::{detector_test_helpers::load_contract, IssueDetector};

    #[test]
    fn test_push_0_opcode_detector_on_0_8_20() {
        let context = load_contract(
            "../tests/contract-playground/out/ExtendedInheritance.sol/ExtendedInheritance.json",
        );

        let mut detector = super::PushZeroOpcodeDetector::default();
        let found = detector.detect(&context).unwrap();
        // assert that it found something
        assert!(found);
        // assert that the number of instances is correct
        assert_eq!(detector.instances().len(), 1);
        // assert that the severity is low
        assert_eq!(
            detector.severity(),
            crate::detect::detector::IssueSeverity::Low
        );
        // assert that the title is correct
        assert_eq!(
            detector.title(),
            String::from("PUSH0 is not supported by all chains")
        );
        // assert that the description is correct
        assert_eq!(
            detector.description(),
            String::from(
                "Solc compiler version 0.8.20 switches the default target EVM version to Shanghai, which means that the generated bytecode will include PUSH0 opcodes. Be sure to select the appropriate EVM version in case you intend to deploy on a chain other than mainnet like L2 chains that may not support PUSH0, otherwise deployment of your contracts will fail."
            )
        );
    }

    #[test]
    fn test_push_0_opcode_detector_on_range() {
        let context =
            load_contract("../tests/contract-playground/out/CrazyPragma.sol/CrazyPragma.json");

        let mut detector = super::PushZeroOpcodeDetector::default();
        let found = detector.detect(&context).unwrap();
        // assert that it found something
        assert!(found);
        // assert that the number of instances is correct
        assert_eq!(detector.instances().len(), 1);
        // assert that the severity is low
        assert_eq!(
            detector.severity(),
            crate::detect::detector::IssueSeverity::Low
        );
        // assert that the title is correct
        assert_eq!(
            detector.title(),
            String::from("PUSH0 is not supported by all chains")
        );
        // assert that the description is correct
        assert_eq!(
            detector.description(),
            String::from(
                "Solc compiler version 0.8.20 switches the default target EVM version to Shanghai, which means that the generated bytecode will include PUSH0 opcodes. Be sure to select the appropriate EVM version in case you intend to deploy on a chain other than mainnet like L2 chains that may not support PUSH0, otherwise deployment of your contracts will fail."
            )
        );
    }

    #[test]
    fn test_push_0_opcode_detector_on_0_8_19() {
        let context = load_contract(
            "../tests/contract-playground/out/ArbitraryTransferFrom.sol/ArbitraryTransferFrom.json",
        );

        let mut detector = super::PushZeroOpcodeDetector::default();
        let found = detector.detect(&context).unwrap();
        // assert that it found something
        assert!(!found);
        // assert that the number of instances is correct
        assert_eq!(detector.instances().len(), 0);
    }

    #[test]
    fn test_push_0_opcode_detector_on_caret_0_8_13() {
        let context =
            load_contract("../tests/contract-playground/out/Counter.sol/Counter.0.8.25.json");

        let mut detector = super::PushZeroOpcodeDetector::default();
        let found = detector.detect(&context).unwrap();
        // assert that it found something
        assert!(found);
        // assert that the number of instances is correct
        assert_eq!(detector.instances().len(), 1);
    }

    #[test]
    fn test_push_0_opcode_detector_on_greter_equal_0_8_0() {
        let context = load_contract(
            "../tests/contract-playground/out/IContractInheritance.sol/IContractInheritance.json",
        );

        let mut detector = super::PushZeroOpcodeDetector::default();
        let found = detector.detect(&context).unwrap();
        // assert that it found something
        assert!(found);
        // assert that the number of instances is correct
        assert_eq!(detector.instances().len(), 1);
    }
}
