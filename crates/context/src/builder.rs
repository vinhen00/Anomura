use std::collections::HashMap;

use crate::{
    ConditionDoublePointer, ConditionalEdge, Edge, GlobalContext, MockHead, MockId, MockState,
    Node, NodeIndex, Nodes, ReturnValDoublePointer, SequenceHead, SequenceHeadIndex, SequenceState,
    Slices,
    errors::{PredicateResult, Result},
    time_mod::TimeModifier,
};

pub struct MockBuilder {
    head: NodeIndex,
    start_index: NodeIndex,
    default_return: Option<ReturnValDoublePointer>,
}
pub struct SequenceBuilder {
    seq_head_index: SequenceHeadIndex,
    effected_mocks: Vec<MockId>,
    enter_sequence: NodeIndex,
    current_index: NodeIndex,
}
impl SequenceBuilder {
    pub fn to_sequence_head(self) -> SequenceHead {
        SequenceHead {
            seq_head_index: self.seq_head_index,
            effected_mocks: self.effected_mocks,
            sequence_state: SequenceState::Inactive,
            node_index: self.enter_sequence,
            enter_sequence: self.enter_sequence,
            exit_sequence: self.current_index,
        }
    }
}
pub struct SliceRef(usize);
pub struct ContextBuilder {
    slices: Slices,
    sequences: Vec<SequenceBuilder>,
    mocks: HashMap<MockId, MockBuilder>,
    nodes: Nodes,
    edges: Vec<Edge>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            slices: Slices::new(),
            sequences: vec![],
            mocks: [].into(),
            nodes: Nodes::new(),
            edges: [].into(),
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
            sequences: self
                .sequences
                .into_iter()
                .map(|s| s.to_sequence_head())
                .collect(),
            nodes: self.nodes,
            edges: self.edges,
        }
    }
    pub fn add_mock<Input, ReturnVal>(
        &mut self,
        mock_id: MockId,
        default_return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
    ) -> Result<()> {
        let index = self.graph.add_node(Node {
            ids: [mock_id.clone()].into(),
            node_kind: crate::NodeKind::Mock,
        });
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
        slice_start: NodeIndex,
        slice_end: NodeIndex,
        mock_id: &MockId,
        condition: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
        return_val_closure: Option<Box<dyn Fn(Input) -> ReturnVal>>,
        modifier: TimeModifier,
    ) -> Result<NodeIndex> {
        let return_val_as_ptr =
            return_val_closure.map(|closure| ReturnValDoublePointer::from_fn(closure));
        let condition_as_ptr = ConditionDoublePointer::from_fn(condition);
        let Some(builder) = self.mocks.get_mut(mock_id) else {
            return Err(format!("mock with id {mock_id:?} does not exist").into());
        };
        let new_node_index = self.graph.add_node(Node {
            ids: [mock_id.clone()].into(),
            node_kind: crate::NodeKind::Mock,
        });
        match modifier {
            TimeModifier::Once => {
                // add a single edge between nodes
                //       condition
                // (1) ---------------> (2)
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    condition: condition_as_ptr,
                    return_val: return_val_as_ptr,
                });
                //consume input edge
                self.graph.add_edge(slice_end, new_node_index, main_weight);
                builder.head = new_node_index;
                Ok(new_node_index)
            }
            TimeModifier::AtMostOnce => {
                //add two edges to new node, one with always,  one with condition)
                //      epsilon || condition
                // (1) ----------------------> (2)
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    condition: condition_as_ptr.clone(),
                    return_val: return_val_as_ptr.clone(),
                });
                let instant_weight = Edge::Instant { priority: 0 };
                //consume input edge
                self.graph.add_edge(slice_end, new_node_index, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(slice_end, new_node_index, instant_weight);
                builder.head = new_node_index;
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
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    condition: condition_as_ptr.clone(),
                    return_val: return_val_as_ptr.clone(),
                });
                let instant_weight = Edge::Instant { priority: 0 };
                //consume input edge
                self.graph.add_edge(slice_end, slice_start, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(slice_end, new_node_index, instant_weight);
                builder.head = new_node_index;
                Ok(new_node_index)
            }
            TimeModifier::AtLeastOnce => {
                //add two edges, one from Node n to n and one instant edge to edge n+1
                //
                //                 condition
                //                   /  \
                //                  |    |
                //     condition     \  /         epsilon
                //  (n) ----------> (n+1) ------------------> (n+2)

                let n_plus_one = self.graph.add_node(Node {
                    node_kind: crate::NodeKind::Mock,
                    ids: [mock_id.clone()].into(),
                });
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    condition: condition_as_ptr.clone(),
                    return_val: return_val_as_ptr.clone(),
                });

                let instant_weight = Edge::Instant { priority: 0 };
                //once
                self.graph
                    .add_edge(slice_end, n_plus_one, main_weight.clone());

                //consume input edge
                self.graph.add_edge(n_plus_one, n_plus_one, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(n_plus_one, new_node_index, instant_weight);
                builder.head = new_node_index;
                Ok(new_node_index)
            }
            TimeModifier::Until(env_id) => todo!(),
            TimeModifier::Times(times_value) => todo!(),
            TimeModifier::After(env_id) => todo!(),
        }
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
