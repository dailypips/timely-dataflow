extern crate timely;
extern crate columnar;

use std::fmt::Debug;
use std::hash::Hash;

use timely::communication::{Data, Communicator, ThreadCommunicator};
use timely::progress::timestamp::RootTimestamp;
use timely::progress::nested::Summary::Local;
use timely::example_static::*;

use columnar::Columnar;

fn main() {
    _distinct(ThreadCommunicator);
}

fn _distinct<C: Communicator>(communicator: C) {

    let mut root = GraphRoot::new(communicator);

    let (mut input1, mut input2) = {

        // allocate a new graph builder
        let mut graph = root.new_subgraph();

        // try building some input scopes
        let (input1, stream1) = graph.new_input::<u64>();
        let (input2, stream2) = graph.new_input::<u64>();

        // prepare some feedback edges
        let (loop1_source, loop1) = graph.loop_variable(RootTimestamp::new(100), Local(1));
        let (loop2_source, loop2) = graph.loop_variable(RootTimestamp::new(100), Local(1));

        let concat1 = (&mut graph).concatenate(vec![stream1, loop1]).disable();
        let concat2 = (&mut graph).concatenate(vec![stream2, loop2]).disable();

        // build up a subgraph using the concatenated inputs/feedbacks
        let (egress1, egress2) = create_subgraph(&mut graph, &concat1, &concat2);

        // connect feedback sources. notice that we have swapped indices ...
        egress1.enable(&mut graph).connect_loop(loop2_source);
        egress2.enable(&mut graph).connect_loop(loop1_source);

        (input1, input2)
    };

    root.step();

    // move some data into the dataflow graph.
    input1.send_at(0, 0..10);
    input2.send_at(0, 1..11);

    // see what everyone thinks about that ...
    root.step();

    input1.advance_to(1000000);
    input2.advance_to(1000000);
    input1.close();
    input2.close();

    // spin
    while root.step() { }
}

fn create_subgraph<G: GraphBuilder, D>(builder: &mut G,
                                        source1: &Stream<G::Timestamp, D>,
                                        source2: &Stream<G::Timestamp, D>) ->
                                            (Stream<G::Timestamp, D>, Stream<G::Timestamp, D>)
where D: Data+Hash+Eq+Debug+Columnar, G::Timestamp: Hash {

    let mut subgraph = builder.new_subgraph::<u64>();

    (subgraph.enter(source1).leave(),
     subgraph.enter(source2).leave())
}
