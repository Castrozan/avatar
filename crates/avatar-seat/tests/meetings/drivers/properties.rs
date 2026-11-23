use std::path::PathBuf;

use avatar_seat::meetings::drivers::{DriverEdges, DriverPlan, install_call_audio_drivers};
use proptest::prelude::*;
use serde_json::{Value, json};

use super::store::NamedStore;
use crate::fakes::output::RecordedOutput;
use crate::fakes::processes::{Log, ScriptedProcesses};

const NAMES: [&str; 4] = ["A.driver", "B.driver", "C.driver", "D.driver"];

#[derive(Debug, Clone)]
struct Slot {
    present: bool,
    identical: bool,
}

fn slot_strategy() -> impl Strategy<Value = Slot> {
    (any::<bool>(), any::<bool>()).prop_map(|(present, identical)| Slot { present, identical })
}

fn missing(slot: &Slot) -> bool {
    !slot.present || !slot.identical
}

fn diff_runs(slots: &[Slot]) -> Vec<Value> {
    let compared = slots.iter().filter(|slot| slot.present);
    let codes = compared.map(|slot| json!({"code": i32::from(!slot.identical)}));
    codes.chain([json!({"code": 0})]).collect()
}

fn plan_for(count: usize) -> DriverPlan {
    let paths = NAMES
        .iter()
        .take(count)
        .map(|name| PathBuf::from(format!("pkg/{name}")));
    DriverPlan {
        directory: PathBuf::from("installed"),
        packaged: paths.collect(),
    }
}

fn installed_names(slots: &[Slot]) -> Vec<String> {
    let pairs = NAMES.iter().zip(slots);
    pairs
        .filter(|(_, slot)| slot.present)
        .map(|(name, _)| (*name).to_owned())
        .collect()
}

fn restarts(log: &Log) -> usize {
    let calls = log.borrow().clone();
    calls
        .iter()
        .filter(|call| call.as_str().is_some_and(|line| line.contains("killall")))
        .count()
}

proptest! {
    #[test]
    fn the_audio_server_restarts_once_exactly_when_something_was_missing(
        slots in proptest::collection::vec(slot_strategy(), 4),
        count in 0_usize..=4,
    ) {
        let considered = &slots[..count];
        let (log, output) = (Log::default(), RecordedOutput::default());
        let processes = ScriptedProcesses::new(&diff_runs(considered), &log, "unused");
        let store = NamedStore::new(&installed_names(&slots));
        let edges = DriverEdges { processes: &processes, store: &store, output: &output };
        prop_assert!(install_call_audio_drivers(&edges, &plan_for(count)).is_ok());
        let lacking: Vec<&str> = NAMES.iter().zip(considered).filter(|(_, slot)| missing(slot)).map(|(name, _)| *name).collect();
        prop_assert_eq!(restarts(&log), usize::from(!lacking.is_empty()));
        let announced = lacking.is_empty() || output.text() == format!("Installed the call audio drivers {}\n", lacking.join(", "));
        prop_assert!(announced);
    }
}
