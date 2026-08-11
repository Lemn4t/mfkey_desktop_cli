use crate::core::engine::DictOutput;
use crate::core::model::MfClassicKey;

pub struct AttackOutcome {
    pub found_keys: Vec<MfClassicKey>,
    pub candidate_total_count: usize,
    pub dict_outputs: Vec<DictOutput>,
}
