// ADERYN-PILOT: 0X01 (Please feel free to fix above imports if they mess up)

use std::{fs::OpenOptions, io::BufWriter, path::PathBuf};

/**
 *
 * Why this exists ?
 *  - To refresh the metadata when changes are made to the detectors
 *  - When you generate a new detector it will be added below
 *
 * IMPORTANT
 *  - Do not EVER remove any comments that start with ADERYN-PILOT: 0x
 *  - Do not add any comments of your own, change function definitions, etc
 *  - However, YOU ARE ALLOWED to modify the custom_detectors array so long as you maintain the original structure.
 */
// ADERYN-PILOT: 0x02 BASIC IMPORTS
use aderyn_driver::detector::{Detector, IssueSeverity};
use serde::Serialize;

// ADERYN-PILOT: 0x03 fn custom_detectors
fn custom_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        // ADERYN-PILOT: 0x04 CUSTOM DETECTORS - Do not remove this comment even if the array is empty
    ]
}

pub fn refresh_metadata() {
    let metadata: Metadata = custom_detectors().into();
    let path = PathBuf::from("metadata/custom_bots.json");
    _ = std::fs::remove_file(&path); // OK to fail

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .unwrap();

    let bw = BufWriter::new(file);

    let value = serde_json::to_value(metadata).unwrap();
    _ = serde_json::to_writer_pretty(bw, &value);
}

impl From<Vec<Box<dyn Detector>>> for Metadata {
    fn from(detectors: Vec<Box<dyn Detector>>) -> Self {
        let mut custom_bots = vec![];
        for detector in detectors {
            let custom_bot = CustomBot {
                title: detector.title(),
                severity: match detector.severity() {
                    IssueSeverity::Critical => "Critical",
                    IssueSeverity::High => "High",
                    IssueSeverity::Low => "Low",
                    IssueSeverity::Medium => "Medium",
                    IssueSeverity::NC => "NC",
                }
                .to_string(),
                description: detector.description(),
            };
            custom_bots.push(custom_bot);
        }
        Metadata { custom_bots }
    }
}

#[derive(Serialize)]
struct Metadata {
    custom_bots: Vec<CustomBot>,
}

#[derive(Serialize)]
struct CustomBot {
    severity: String,
    title: String,
    description: String,
}
