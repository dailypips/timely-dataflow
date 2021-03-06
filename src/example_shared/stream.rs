use example_shared::builder::GraphBuilder;

use progress::Timestamp;
use progress::nested::subgraph::{Source, Target};

use communication::Observer;
use communication::Data;
use communication::output_port::Registrar;

#[derive(Clone)]
pub struct Stream<G: GraphBuilder, D:Data> {
    pub name:   Source,                     // used to name the source in the host graph.
    ports:      Registrar<G::Timestamp, D>, // used to register interest in the output.
    builder:    G,
}

impl<G: GraphBuilder, D:Data> Stream<G, D> {

    pub fn connect_to<O>(&self, target: Target, observer: O)
    where O: Observer<Time=G::Timestamp, Data=D>+'static {
        self.builder.add_edge(self.name, target);
        self.ports.add_observer(observer);
    }

    pub fn new(source: Source, output: Registrar<G::Timestamp, D>, builder: G) -> Self {
        Stream { name: source, ports: output, builder: builder }
    }

    pub fn builder(&self) -> G { self.builder.clone() }
}
