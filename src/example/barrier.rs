use progress::frontier::Antichain;
use progress::{ Scope, CountMap };
use progress::nested::summary::Summary;
use progress::nested::summary::Summary::Local;
use progress::nested::product::Product;
use progress::timestamp::{RootTimestamp, RootSummary};

pub struct BarrierScope {
    pub ready:  bool,
    pub epoch:  u64,
    pub degree: u64,
    pub ttl:    u64,
}

impl Scope<Product<RootTimestamp, u64>> for BarrierScope {
    fn name(&self) -> String { format!("Barrier") }
    fn inputs(&self) -> u64 { 1 }
    fn outputs(&self) -> u64 { 1 }

    fn get_internal_summary(&mut self) -> (Vec<Vec<Antichain<Summary<RootSummary, u64>>>>, Vec<CountMap<Product<RootTimestamp, u64>>>) {
        return (vec![vec![Antichain::from_elem(Local(1))]],
                vec![CountMap::new_from(&Product::new(RootTimestamp, self.epoch), self.degree as i64)]);
    }

    fn set_external_summary(&mut self, _summaries: Vec<Vec<Antichain<Summary<RootSummary, u64>>>>, _frontier: &mut [CountMap<Product<RootTimestamp, u64>>]) -> () {
        for x in _frontier { x.clear(); }
    }

    fn push_external_progress(&mut self, external: &mut [CountMap<Product<RootTimestamp, u64>>]) -> () {
        while let Some((time, val)) = external[0].pop() {
            if time.inner == self.epoch - 1 && val == -1 {
                self.ready = true;
            }
        }
    }

    fn pull_internal_progress(&mut self, internal: &mut [CountMap<Product<RootTimestamp, u64>>],
                                        _consumed: &mut [CountMap<Product<RootTimestamp, u64>>],
                                        _produced: &mut [CountMap<Product<RootTimestamp, u64>>]) -> bool {
        if self.ready {
            internal[0].update(&Product::new(RootTimestamp, self.epoch), -1);

            if self.epoch < self.ttl {
                internal[0].update(&Product::new(RootTimestamp, self.epoch + 1), 1);
            }

            self.epoch += 1;
            self.ready = false;
        }

        return false;
    }
}
