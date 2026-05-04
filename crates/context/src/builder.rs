use std::collections::HashMap;

use crate::{
    ConditionDoublePointer, GlobalContext, ReturnValDoublePointer,
    errors::{PredicateResult, Result},
    time_mod::TimeModifier,
    types::{
        edges::{ConditionalEdge, Edge, EdgeIndex},
        mock::{MockHead, MockId, MockState},
        nodes::{Node, NodeIndex, NodeKind, Nodes},
        sequences::{SequenceHeads, SequenceIndex},
        slices::Slices,
    },
};

pub struct MockBuilder {
    head: NodeIndex,
    start_index: NodeIndex,
    default_return: Option<ReturnValDoublePointer>,
}
pub struct ContextBuilder {
    slices: Slices,
    sequences: SequenceHeads,
    mocks: HashMap<MockId, MockBuilder>,
    nodes: Nodes,
}

impl ContextBuilder {
    ///checks if the sequence with the given index contains the given mock id
    pub fn seq_contains(&mut self, sequence_index: SequenceIndex, id: &MockId) -> bool {
        self.sequences.contains_id(sequence_index, id)
    }

    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        self.nodes.add(node)
    }
    pub fn get_node_mut(&mut self, node_index: NodeIndex) -> Option<&mut Node> {
        self.nodes.get_node_mut(node_index)
    }
    pub fn get_node_ref(&mut self, node_index: NodeIndex) -> Option<&Node> {
        self.nodes.get_node_ref(node_index)
    }
    pub fn add_edge(&mut self, node_index: NodeIndex, edge: Edge) -> Option<EdgeIndex> {
        self.get_node_mut(node_index).map(|n| n.add_edge(edge))
    }
    pub fn new() -> Self {
        Self {
            slices: Slices::new(),
            sequences: SequenceHeads::new(),
            mocks: [].into(),
            nodes: Nodes::new(),
        }
    }

    pub fn finish(self) -> GlobalContext {
        GlobalContext {
            mock_heads: self
                .mocks
                .into_iter()
                .map(|(k, i)| {
                    (
                        k,
                        MockHead {
                            state: MockState::Unlocked {
                                mock_head_index: i.start_index,
                            },
                            default_return_val: i.default_return,
                        },
                    )
                })
                .collect(),
            slices: self.slices,
            sequences: self.sequences,
            nodes: self.nodes,
        }
    }
    /// adds the given mock id to the node with the given index, returns true if successful and false if the node does not exist
    pub fn add_id_to_node(&mut self, node_index: NodeIndex, mock_id: MockId) -> bool {
        self.get_node_mut(node_index)
            .map(|n| n.add_id(mock_id))
            .unwrap_or(false)
    }

    pub fn add_mock<Input, ReturnVal>(
        &mut self,
        mock_id: MockId,
        default_return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
    ) -> Result<()> {
        let index = self.add_node(Node::new(NodeKind::Mock));
        let ptr_return = default_return_val_closure.map(|r| ReturnValDoublePointer::from_fn(r));
        match self.mocks.insert(
            mock_id.clone(),
            MockBuilder {
                head: index,
                start_index: index,
                default_return: ptr_return,
            },
        ) {
            Some(_) => Err(format!("mock {mock_id:?} entered into context twice").into()),
            None => Ok(()),
        }
    }
    pub fn add_expectation<Input, ReturnVal>(
        &mut self,
        mock_id: &MockId,
        condition: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
        return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
        modifier: TimeModifier,
    ) -> Result<()> {
        let starting_point = self.mocks.get(mock_id).unwrap().head;

        let new_index = self.add_expectation_to_starting_point(
            starting_point,
            mock_id,
            condition,
            return_val_closure,
            modifier,
        )?;
        self.mocks
            .entry(mock_id.clone())
            .and_modify(|m| m.head = new_index);
        Ok(())
    }
    pub fn add_expectation_to_starting_point<Input, ReturnVal>(
        &mut self,
        starting_point: NodeIndex,
        mock_id: &MockId,
        condition: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
        return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
        modifier: TimeModifier,
    ) -> Result<NodeIndex> {
        let return_val_as_ptr =
            return_val_closure.map(|closure| ReturnValDoublePointer::from_fn(closure));
        let condition_as_ptr = ConditionDoublePointer::from_fn(condition);
        let create_conditional_edge = |priority, target| {
            Edge::Condition(ConditionalEdge {
                priority,
                condition: condition_as_ptr.to_owned(),
                return_val: return_val_as_ptr.to_owned(),
                target,
            })
        };
        let Some(builder) = self.mocks.get_mut(mock_id) else {
            return Err(format!("mock with id {mock_id:?} does not exist").into());
        };

        let new_node_index = self.add_node(Node::new(NodeKind::Mock));
        self.add_id_to_node(starting_point, mock_id.clone());
        match modifier {
            TimeModifier::Once => {
                // add a single edge between nodes
                //       condition
                // (1) ---------------> (2)
                let main_edge = create_conditional_edge(0, new_node_index);
                //consume input edge

                self.add_edge(starting_point, main_edge).unwrap();
                Ok(new_node_index)
            }
            TimeModifier::AtMostOnce => {
                //add two edges to new node, one with always,  one with condition)
                //      epsilon || condition
                // (1) ----------------------> (2)
                let main_edge = create_conditional_edge(1, new_node_index);
                let instant_edge = Edge::Instant {
                    priority: 0,
                    target: new_node_index,
                };
                //consume input edge
                self.add_edge(starting_point, main_edge).unwrap();
                self.add_edge(starting_point, instant_edge).unwrap();
                //epsilon edge
                Ok(new_node_index)
            }

            TimeModifier::Any => {
                //add two edges, one from Node n to n and one instant edge to edge n+1
                //
                //    condition
                //      /   \
                //      |    |
                //      \   /         epsilon
                //       (1) ------------------> (2)
                let main_weight = create_conditional_edge(1, starting_point);
                let instant_edge = Edge::Instant {
                    priority: 0,
                    target: new_node_index,
                };

                //consume input edge
                self.add_edge(starting_point, main_weight).unwrap();

                //epsilon edge
                self.add_edge(starting_point, instant_edge);
                Ok(new_node_index)
            }
            TimeModifier::AtLeastOnce => {
                //add two edges, one from Node n to n and one instant edge to edge n+1
                //
                //                 condition2
                //                   /  \
                //                  |    |
                //     condition1     \  /         epsilon
                //  (n) ----------> (n+1) ------------------> (n+2)

                let n_plus_one = self.add_node(Node::new(NodeKind::Mock));
                assert!(self.add_id_to_node(n_plus_one, mock_id.clone()));
                self.add_edge(n_plus_one, create_conditional_edge(1, n_plus_one))
                    .unwrap();
                //epsilon
                self.add_edge(
                    n_plus_one,
                    Edge::Instant {
                        priority: 0,
                        target: new_node_index,
                    },
                )
                .unwrap();

                //condition1
                self.add_edge(starting_point, create_conditional_edge(1, n_plus_one));

                Ok(new_node_index)
            }
            TimeModifier::Until(env_id) => todo!(),
            TimeModifier::Times(times_value) => todo!(),
            TimeModifier::After(env_id) => todo!(),
        }
    }

    pub fn add_enter_sequence(
        &mut self,
        sequence_index: SequenceIndex,
        mock_id: &MockId,
        modifier: TimeModifier,
    ) -> Result<()> {
        let starting_point = self.mocks.get(mock_id).unwrap().head;
        self.add_enter_sequence_at_starting_point(
            starting_point,
            sequence_index,
            mock_id,
            modifier,
        )?;
        Ok(())
    }
    pub fn add_enter_sequence_at_starting_point(
        &mut self,
        starting_point: NodeIndex,
        sequence_index: SequenceIndex,
        mock_id: &MockId,
        modifier: TimeModifier,
    ) -> Result<NodeIndex> {
        let new_node_index = self.add_node(Node::new(NodeKind::Mock));
        if self.seq_contains(sequence_index, mock_id) {
            return Err(format!(
                "expected sequence with index{:?} to contain id {:?}",
                sequence_index, mock_id
            )
            .into());
        }
        let exit_node = self
            .sequences
            .edge_ref(sequence_index)
            .unwrap()
            .exit_sequence;
        match modifier {
            TimeModifier::Once => {
                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::AtMostOnce => {
                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: new_node_index,
                    },
                );
                self.add_edge(
                    starting_point,
                    Edge::Instant {
                        priority: 0,
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::Any => {
                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: starting_point,
                    },
                );
                self.add_edge(
                    starting_point,
                    Edge::Instant {
                        priority: 0,
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::AtLeastOnce => {
                let node_plus = self.add_node(Node::new(NodeKind::Mock));

                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(node_plus, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: node_plus,
                    },
                );
                self.add_edge(
                    node_plus,
                    Edge::Instant {
                        priority: 0,
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::Until(env_id) => todo!(),
            TimeModifier::Times(times_value) => todo!(),
            TimeModifier::After(env_id) => todo!(),
        };
        Ok(new_node_index)
    }
}
/*
    pub fn add_slice_to_starting_point(
        &mut self,
        starting_point: NodeIndex,
        slice_ref: SliceRef,
        mock_id: &MockId,
        modifier: TimeModifier,
    ) -> Result<NodeIndex> {
        let new_node_index = self.add_node(Node::new(NodeKind::Mock));
        let slice = self.slices.get_ref_slice(slice_ref).unwrap();
        for node in slice.clone().into_iter() {}
        let exit_node = match modifier {
            TimeModifier::Once => {
                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::AtMostOnce => {
                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: new_node_index,
                    },
                );
                self.add_edge(
                    starting_point,
                    Edge::Instant {
                        priority: 0,
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::Any => {
                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: starting_point,
                    },
                );
                self.add_edge(
                    starting_point,
                    Edge::Instant {
                        priority: 0,
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::AtLeastOnce => {
                let node_plus = self.add_node(Node::new(NodeKind::Mock));

                self.add_edge(starting_point, Edge::SequenceEnter(sequence_index));
                self.add_edge(node_plus, Edge::SequenceEnter(sequence_index));
                self.add_edge(
                    exit_node,
                    Edge::SequenceExit {
                        id: mock_id.clone(),
                        target: node_plus,
                    },
                );
                self.add_edge(
                    node_plus,
                    Edge::Instant {
                        priority: 0,
                        target: new_node_index,
                    },
                );
            }
            TimeModifier::Until(env_id) => todo!(),
            TimeModifier::Times(times_value) => todo!(),
            TimeModifier::After(env_id) => todo!(),
        };
        Ok(new_node_index)
    }
}*/

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
