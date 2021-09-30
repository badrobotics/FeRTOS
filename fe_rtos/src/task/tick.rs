use core::sync::atomic::{AtomicU32, Ordering};

pub(crate) struct TickCounter {
    lsb: AtomicU32,
    msb: AtomicU32,
}

impl TickCounter {
    pub const fn new() -> Self {
        TickCounter {
            lsb: AtomicU32::new(0),
            msb: AtomicU32::new(0),
        }
    }

    pub fn inc(&self) {
        let old_lsb = self.lsb.fetch_add(1, Ordering::SeqCst);

        //If there was overflow, increment the lsb
        if self.lsb.load(Ordering::SeqCst) < old_lsb {
            let old_msb = self.msb.load(Ordering::SeqCst);
            self.msb
                .compare_exchange(old_msb, old_msb + 1, Ordering::SeqCst, Ordering::SeqCst)
                .ok();
        }
    }

    pub fn get(&self) -> u64 {
        let lsb = self.lsb.load(Ordering::SeqCst);
        let msb = self.msb.load(Ordering::SeqCst);

        ((msb as u64) << 32) | (lsb as u64)
    }
}
