
#[derive(Debug, Default, Clone)]
pub struct ReorgState {
    pub last_finalized_slot: u64,
}

impl ReorgState {
    pub fn observe_finalized(&mut self, slot: u64) {
        self.last_finalized_slot = self.last_finalized_slot.max(slot);
    }
}
