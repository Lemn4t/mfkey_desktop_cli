use crate::core::engine::{self, SaveDictFn};
use crate::core::model::Nonce;
use crate::core::outcome::AttackOutcome;
use crate::core::reporter::Reporter;
use crate::core::state::AttackState;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct Crapto1Solver;

impl Crapto1Solver {
    pub fn run(
        reporter: Arc<dyn Reporter>,
        stop: Arc<AtomicBool>,
        nonces: &[Nonce],
        dict_output_dir: Option<&str>,
        save_dict: &mut SaveDictFn,
    ) -> AttackOutcome {
        let mut state = AttackState::new(reporter, stop);

        let (candidate_total_count, dict_outputs) =
            engine::run_attack(&mut state, nonces, dict_output_dir, save_dict);

        AttackOutcome {
            found_keys: state.found_keys.into_vec(),
            candidate_total_count,
            dict_outputs,
        }
    }
}
