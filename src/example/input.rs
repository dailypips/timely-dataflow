use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;

use progress::frontier::{MutableAntichain, Antichain};
use progress::{Scope, Timestamp};
use progress::nested::subgraph::Source::{ScopeOutput};
use progress::count_map::CountMap;

use communication::*;
use communication::channels::ObserverHelper;
use example::builder::Graph;
use example::stream::Stream;

// TODO : This is an exogenous input, but it would be nice to wrap a Subgraph in something
// TODO : more like a harness, with direct access to its inputs.

// NOTE : This only takes a &self, not a &mut self, which works but is a bit weird.
// NOTE : Experiments with &mut indicate that the borrow of 'a lives for too long.
// NOTE : Might be able to fix with another lifetime parameter, say 'c: 'a.

// returns both an input scope and a stream representing its output.
pub trait InputExtensionTrait<G: Graph> {
    fn new_input<'a, D:Data>(&'a self) -> (InputHelper<G::Timestamp, D>, Stream<'a, G, D>) where G: 'a;
}

impl<G: Graph> InputExtensionTrait<G> for RefCell<G> {
    fn new_input<'a, D:Data>(&'a self) -> (InputHelper<G::Timestamp, D>, Stream<'a, G, D>) where G: 'a {
        let (output, registrar) = OutputPort::<G::Timestamp, D>::new();
        let produced = Rc::new(RefCell::new(CountMap::new()));

        let helper = InputHelper {
            frontier: Rc::new(RefCell::new(MutableAntichain::new_bottom(Default::default()))),
            progress: Rc::new(RefCell::new(CountMap::new())),
            output:   ObserverHelper::new(output, produced.clone()),
        };

        let copies = self.borrow_mut().with_communicator(|x| x.peers());

        let index = self.borrow_mut().add_scope(InputScope {
            frontier: helper.frontier.clone(),
            progress: helper.progress.clone(),
            messages: produced.clone(),
            copies:   copies,
        });

        return (helper, Stream::new(ScopeOutput(index, 0), registrar, self));
    }
}

pub struct InputScope<T:Timestamp> {
    frontier:   Rc<RefCell<MutableAntichain<T>>>,   // times available for sending
    progress:   Rc<RefCell<CountMap<T>>>,           // times closed since last asked
    messages:   Rc<RefCell<CountMap<T>>>,           // messages sent since last asked
    copies:     u64,
}

impl<T:Timestamp> Scope<T> for InputScope<T> {
    fn name(&self) -> String { format!("Input") }
    fn inputs(&self) -> u64 { 0 }
    fn outputs(&self) -> u64 { 1 }

    fn get_internal_summary(&mut self) -> (Vec<Vec<Antichain<T::Summary>>>, Vec<CountMap<T>>) {
        let mut map = CountMap::new();
        for x in self.frontier.borrow().elements.iter() {
            map.update(x, self.copies as i64);
        }
        (Vec::new(), vec![map])
    }

    fn pull_internal_progress(&mut self, frontier_progress: &mut [CountMap<T>],
                                        _messages_consumed: &mut [CountMap<T>],
                                         messages_produced: &mut [CountMap<T>]) -> bool
    {
        self.messages.borrow_mut().drain_into(&mut messages_produced[0]);
        self.progress.borrow_mut().drain_into(&mut frontier_progress[0]);
        return false;
    }

    fn notify_me(&self) -> bool { false }
}

pub struct InputHelper<T: Timestamp, D: Data> {
    frontier:   Rc<RefCell<MutableAntichain<T>>>,   // times available for sending
    progress:   Rc<RefCell<CountMap<T>>>,           // times closed since last asked
    output:     ObserverHelper<OutputPort<T, D>>,
}

impl<T:Timestamp, D: Data> InputHelper<T, D> {
    pub fn send_messages(&mut self, time: &T, data: Vec<D>) {
        self.output.open(time);
        for datum in data.into_iter() { self.output.give(datum); }
        self.output.shut(time);
    }

    pub fn advance(&self, start: &T, end: &T) {
        self.frontier.borrow_mut().update_weight(start, -1, &mut (*self.progress.borrow_mut()));
        self.frontier.borrow_mut().update_weight(end,  1, &mut (*self.progress.borrow_mut()));
    }

    pub fn close_at(&self, time: &T) {
        self.frontier.borrow_mut().update_weight(time, -1, &mut (*self.progress.borrow_mut()));
    }
}
