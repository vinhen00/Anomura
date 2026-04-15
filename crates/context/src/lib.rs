use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    result,
    sync::{Mutex, OnceLock},
};

use derive_more::{AsMut, AsRef, Display, FromStr};
use petgraph::{
    Graph,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use proc_macro2::TokenStream;
use quote::quote;

pub static GLOBAL_CONTEXT: OnceLock<Mutex<GlobalContext>> = OnceLock::new();
#[derive(Debug, Clone)]
pub enum SequenceState {
    Inactive,
    Active,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceHeadIndex(usize);
#[derive(Clone, Debug)]
pub struct SequenceHead {
    seq_head_index: SequenceHeadIndex,
    effected_mocks: Vec<MockId>,
    sequence_state: SequenceState,
    node_index: NodeIndex,
    enter_sequence: NodeIndex,
    exit_sequence: NodeIndex,
}
#[derive(Clone, Debug)]
pub enum MockState {
    Locked {
        sequence_head_index: SequenceHeadIndex,
    },
    Unlocked {
        mock_head_index: NodeIndex<u32>,
    },
}
pub struct MockHead {
    state: MockState,
    default_return_val: Option<ReturnValDoublePointer>,
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";
#[derive(Debug, Clone, Hash, FromStr, AsRef, AsMut, core::cmp::Eq, PartialEq)]
pub struct MockId(String);
impl MockId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

pub struct GlobalContext {
    sequence_heads: Vec<SequenceHead>,

    graph: DiGraph<MockNode, Edge>,

    mock_heads: HashMap<MockId, MockHead>,
}

pub struct EdgeTransitionInfo {
    priority: u8,
    return_val: Option<ReturnValDoublePointer>,
    target_node: NodeIndex,
}

impl GlobalContext {
    pub fn get_dot(&self) -> String {
        let dot = petgraph::dot::Dot::new(&self.graph);
        format!("{dot:?}")
    }
    pub fn get_sequence_head(&self, index: SequenceHeadIndex) -> &SequenceHead {
        &self.sequence_heads[index.0]
    }
    pub fn mut_sequence_head(&mut self, index: SequenceHeadIndex) -> &mut SequenceHead {
        &mut self.sequence_heads[index.0]
    }

    pub fn enter_sequence(&mut self, seq_head_index: SequenceHeadIndex) -> Result<()> {
        let seq_head_vec = self
            .get_sequence_head(seq_head_index)
            .effected_mocks
            .clone();
        for id in &seq_head_vec {
            let mock_head_index = match self.mock_heads.get(id).map(|h| &h.state) {
                Some(MockState::Locked {
                    sequence_head_index,
                }) => {
                    return Err(format!(
                        "mock with id {:?} wanted to enter sequence with index {:?} 
                        but it was already in sequence with index{:?}",
                        id, seq_head_index, sequence_head_index
                    )
                    .into());
                }
                Some(MockState::Unlocked { mock_head_index }) => *mock_head_index,
                None => {
                    return Err(format!(
                        "mock_id {:?} not found when trying to enter sequence with index {:?}",
                        id, seq_head_index
                    )
                    .into());
                }
            };
            let mut instant_stack: Vec<NodeIndex> = vec![mock_head_index];
            let mut visited = HashSet::new();
            let mut sequence_node_found = false;
            while !sequence_node_found && let Some(node_index) = instant_stack.pop() {
                assert!(
                    self.graph.node_weight(node_index).is_some(),
                    "no node with id {:?} found when trying to enter sequence with index {:?}",
                    &id,
                    seq_head_index
                );

                sequence_node_found = self
                    .graph
                    .edges_directed(node_index, petgraph::Direction::Outgoing)
                    .any(|e| match e.weight() {
                        Edge::SequenceEnter(sequence_head_index)
                            if *sequence_head_index == seq_head_index =>
                        {
                            assert!(
                                e.target()
                                    == self.get_sequence_head(*sequence_head_index).enter_sequence
                            );
                            true
                        }
                        Edge::Instant { .. } => {
                            let target = e.target();
                            if !visited.contains(&target) {
                                instant_stack.push(e.target());
                                visited.insert(e.target());
                            }
                            false
                        }
                        _ => false,
                    });
            }
            if !sequence_node_found {
                return Err(format!(
                    "Mock with id {:?} couldn't find a valid entry point for entering sequence{:?}",
                    id, seq_head_index
                )
                .into());
            }
        }
        // All related mock was in the correct position, so we move their heads and return success
        for id in seq_head_vec {
            self.mock_heads.entry(id.clone()).and_modify(|h| {
                h.state = MockState::Locked {
                    sequence_head_index: seq_head_index,
                }
            });
        }
        Ok(())
    }

    pub fn run_mock<Input, ReturnVal>(
        &mut self,
        mock_id: MockId,
        input: &Input,
    ) -> Result<ReturnVal> {
        let (node_index, maybe_sequence_head) = match self
            .mock_heads
            .get(&mock_id)
            .map(|mock_head| &mock_head.state)
        {
            Some(MockState::Locked {
                sequence_head_index,
            }) => {
                let seq_head = self.get_sequence_head(*sequence_head_index).clone();
                (seq_head.node_index, Some(seq_head))
            }
            Some(MockState::Unlocked { mock_head_index }) => (*mock_head_index, None),
            None => return Err(MockError::NoMatchingId),
        };
        let mut sequence_head_indices = vec![];
        let mut sequence_stack_append = vec![];
        let mut instant_stack_append = vec![];
        let mut node_index_stack = vec![node_index];
        let mut visited = vec![];
        let mut failed_conditionals: Vec<PredicateError> = vec![];
        let mut successful_conditionals: Vec<_> = vec![];
        //traverses graph until
        while let Some(node_index) = node_index_stack.pop() {
            let Some(res) = self.graph.node_weight(node_index) else {
                return Err("node not found".into());
            };
            if res.id != mock_id {
                return Err(format!(
                    "node with index {:?} expected {:?} but received {:?}",
                    node_index, mock_id, res.id
                )
                .into());
            };
            self.graph
                .edges_directed(node_index, petgraph::Direction::Outgoing)
                .for_each(|e| match e.weight() {
                    Edge::Instant { .. } => instant_stack_append.push(e.target()),
                    Edge::Condition(conditional_edge) => {
                        let condition = unsafe { conditional_edge.condition.into_fn::<Input>() };
                        let res = condition(input);
                        match res {
                            Ok(()) => successful_conditionals.push(EdgeTransitionInfo {
                                priority: conditional_edge.priority,
                                return_val: conditional_edge.return_val.clone(),
                                target_node: e.target(),
                            }),
                            Err(e) => {
                                failed_conditionals.push(e);
                            }
                        }
                    }
                    Edge::SequenceEnter(index) => sequence_head_indices.push((node_index, *index)),
                    Edge::SequenceExit(exit_mock_id)
                        if maybe_sequence_head
                            .clone()
                            .map(|head| head.effected_mocks.contains(exit_mock_id))
                            .unwrap_or(false) =>
                    {
                        if *exit_mock_id == mock_id {
                            sequence_stack_append.push(e.target())
                        }
                        self.mock_heads
                            .entry(exit_mock_id.clone())
                            .and_modify(|head| {
                                head.state = MockState::Unlocked {
                                    mock_head_index: e.target(),
                                }
                            });
                    }
                    _ => {}
                });
            for (target, seq_head) in sequence_head_indices.drain(0..) {
                if let Ok(()) = self.enter_sequence(seq_head) {
                    sequence_stack_append.push(target)
                }
            }
            //if valid conditions were found, pick the edge with the highest priority
            if !successful_conditionals.is_empty() {
                successful_conditionals.sort_by(|e, f| e.priority.cmp(&f.priority));
                let edge = successful_conditionals
                    .last()
                    .expect("we have already checked that edges is not empty");
                let return_val_ptr = edge.return_val.as_ref().unwrap_or(
                    self.mock_heads[&mock_id]
                        .default_return_val
                        .as_ref()
                        .expect("no return value found"),
                );
                let return_val = unsafe { return_val_ptr.into_fn::<ReturnVal>()() };
                let new_node_index = edge.target_node;
                if self.graph.node_weight(new_node_index).is_none() {
                    return Err(format!("new node index {:?} is invalid", new_node_index).into());
                };
                self.mock_heads.entry(mock_id.clone()).and_modify(|h| {
                    h.state = MockState::Unlocked {
                        mock_head_index: new_node_index,
                    }
                });
                return Ok(return_val);
            }
            node_index_stack.append(&mut instant_stack_append);
            node_index_stack.append(&mut sequence_stack_append);
            visited.push(node_index);
        }
        //failed to find any valid conditions
        let joined = failed_conditionals
            .into_iter()
            .map(|e| e.0)
            .collect::<Vec<_>>()
            .join(",\n");
        Err(format!(
            "No matching condition found for input tried the following:  {}",
            joined
        )
        .into())
    }
}
/*
an expectation can be a series of expectations,
There is a pool of mocks moving through the graph independently at the start.
If mock A enters a sequence where B is also dependent, A can take temporary ownership of B, assuming B is in a state where this is acceptable.
//if B is not in such a state, for example when moving through another sequence
*/

pub struct ExpectationRef(u32);

#[derive(Debug, Clone, Display, FromStr)]
pub struct EnvId(String);
pub enum EnvVal {
    ExpectationRef(ExpectationRef),
    Int(u32),
}
pub type Environment = HashMap<EnvId, EnvVal>;
pub struct MockBuilder {
    head: NodeIndex,
    start_index: NodeIndex,
    default_return: Option<ReturnValDoublePointer>,
}
#[derive(Debug, Clone)]
pub struct ReturnValDoublePointer {
    thin_ptr: *const (),
}
impl ReturnValDoublePointer {
    pub fn from_fn<ReturnVal>(closure: Box<dyn Fn() -> ReturnVal>) -> Self {
        let cloref = Box::leak(closure);
        let wrapped = Box::new(cloref);
        let wraref = Box::into_raw(wrapped);
        let thin_ptr = wraref as *const _;
        Self { thin_ptr }
    }
    /// Casts the a raw double pointer created with from_fn into a closure ` Result<&dyn Fn() -> ReturnVal>`
    /// # Safety
    /// You must guarantee that `ReturnVal` is the exact same Type as was used when you used `from_fn` to create the value.
    pub unsafe fn into_fn<ReturnVal>(&self) -> &dyn Fn() -> ReturnVal {
        let wraref: *mut &mut dyn Fn() -> ReturnVal = self.thin_ptr as _;
        let cloref: &mut dyn Fn() -> ReturnVal = unsafe { *wraref };
        cloref
    }
}

#[derive(Debug, Clone)]
pub struct ConditionDoublePointer {
    thin_ptr: *const (),
}
impl ConditionDoublePointer {
    pub fn from_fn<Input>(closure: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>) -> Self {
        let cloref = Box::leak(closure);
        let wrapped = Box::new(cloref);
        let wraref = Box::into_raw(wrapped);
        let thin_ptr = wraref as *const _;
        Self { thin_ptr }
    }
    /// Casts the a raw double pointer created with from_fn into a closure with type `Fn(&Input) -> Result<()>`
    /// # Safety
    /// You must guarantee that `Input` is the exact same Type as was used when you used `from_fn` to create the value.
    pub unsafe fn into_fn<Input>(&self) -> &dyn Fn(&Input) -> PredicateResult<()> {
        let wraref: *mut &mut dyn Fn(&Input) -> PredicateResult<()> = self.thin_ptr as _;
        let cloref: &mut dyn Fn(&Input) -> PredicateResult<()> = unsafe { *wraref };
        cloref
    }
}
//Our DoublePointers will live through the remainder of the program, and they will not be modified in any way.
unsafe impl Send for ConditionDoublePointer {}
unsafe impl Sync for ConditionDoublePointer {}
unsafe impl Send for ReturnValDoublePointer {}
unsafe impl Sync for ReturnValDoublePointer {}

pub struct ContextBuilder {
    mocks: HashMap<MockId, MockBuilder>,
    pub(crate) graph: DiGraph<MockNode, Edge>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        let graph = Graph::new();
        Self {
            graph,
            mocks: HashMap::new(),
        }
    }

    pub fn finish(self) -> GlobalContext {
        GlobalContext {
            sequence_heads: vec![],
            graph: self.graph,
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
        }
    }
    pub fn add_mock<ReturnVal>(
        &mut self,
        mock_id: MockId,
        default_return_val_closure: Option<Box<dyn Fn() -> ReturnVal>>,
    ) -> Result<()> {
        let index = self.graph.add_node(MockNode {
            entry: true,
            exit: false,
            id: mock_id.clone(),
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
        mock_id: &MockId,
        condition: Box<dyn Fn(&Input) -> PredicateResult<()> + 'static>,
        return_val_closure: Option<Box<dyn Fn() -> ReturnVal>>,
        modifier: TimeModifier,
        exit: bool,
    ) -> Result<()> {
        let return_val_as_ptr =
            return_val_closure.map(|closure| ReturnValDoublePointer::from_fn(closure));
        let condition_as_ptr = ConditionDoublePointer::from_fn(condition);
        let Some(builder) = self.mocks.get_mut(mock_id) else {
            return Err(format!("mock with id {mock_id:?} does not exist").into());
        };
        let new_node_index = self.graph.add_node(MockNode {
            entry: false,
            exit,
            id: mock_id.clone(),
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
                self.graph
                    .add_edge(builder.head, new_node_index, main_weight);
                builder.head = new_node_index;
                Ok(())
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
                self.graph
                    .add_edge(builder.head, new_node_index, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(builder.head, new_node_index, instant_weight);
                builder.head = new_node_index;
                Ok(())
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
                self.graph.add_edge(builder.head, builder.head, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(builder.head, new_node_index, instant_weight);
                builder.head = new_node_index;
                Ok(())
            }
            TimeModifier::AtLeastOnce => {
                //add two edges, one from Node n to n and one instant edge to edge n+1
                //
                //                 condition
                //                   /  \
                //                  |    |
                //     condition     \  /         epsilon
                //  (n) ----------> (n+1) ------------------> (n+2)

                let n_plus_one = self.graph.add_node(MockNode {
                    entry: false,
                    exit,
                    id: mock_id.clone(),
                });
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    condition: condition_as_ptr.clone(),
                    return_val: return_val_as_ptr.clone(),
                });

                let instant_weight = Edge::Instant { priority: 0 };
                //once
                self.graph
                    .add_edge(builder.head, n_plus_one, main_weight.clone());

                //consume input edge
                self.graph.add_edge(n_plus_one, n_plus_one, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(n_plus_one, new_node_index, instant_weight);
                builder.head = new_node_index;
                Ok(())
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

#[derive(Display, Debug, Clone)]
pub enum TimesValue {
    Explicit(u32),
    Implicit(EnvId),
}
#[derive(Display, Debug, Clone)]
pub enum TimeModifier {
    Once,
    AtMostOnce,
    Any,
    AtLeastOnce,
    Until(EnvId),
    Times(TimesValue),
    After(EnvId),
}

impl quote::ToTokens for TimeModifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let append = match self {
            TimeModifier::Once => quote! { context::TimeModifier::Once },
            TimeModifier::AtMostOnce => quote! { context::TimeModifier::AtMostOnce },
            TimeModifier::Any => quote! { context::TimeModifier::Any },
            TimeModifier::AtLeastOnce => quote! { context::TimeModifier::AtLeastOnce },
            TimeModifier::Until(env_id) => quote! { context::TimeModifier::Until },
            TimeModifier::Times(times_value) => quote! { context::TimeModifier::Times},
            TimeModifier::After(env_id) => quote! { context::TimeModifier::After },
        };
        tokens.extend(append);
    }
}

#[derive(Debug, Clone)]
pub struct ConditionalEdge {
    priority: u8,
    condition: ConditionDoublePointer,
    return_val: Option<ReturnValDoublePointer>,
}

#[derive(Debug, Clone)]
pub struct SequenceExit {
    priority: u8,
    id: MockId,
}
#[derive(Debug, Clone)]
pub enum Edge {
    Instant { priority: u8 },
    Condition(ConditionalEdge),
    SequenceEnter(SequenceHeadIndex),
    SequenceExit(MockId),
}
#[derive(Debug, Clone)]
pub enum Node {
    MockNode(MockNode),
    SequenceNode(SequenceNode),
}
#[derive(Debug, Clone)]
pub struct MockNode {
    entry: bool,
    exit: bool,
    id: MockId,
}

#[derive(Debug, Clone)]
pub struct SequenceNode {
    entry: bool,
    exit: bool,
    id: MockId,
    ids: Vec<MockId>,
}
#[derive(Clone, Debug, Display)]
pub struct PredicateError(pub String);
impl From<&str> for PredicateError {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
impl From<String> for PredicateError {
    fn from(value: String) -> Self {
        Self(value)
    }
}
pub type PredicateResult<T> = result::Result<T, PredicateError>;

pub type Result<T> = result::Result<T, MockError>;
#[derive(Debug, Clone, Display)]
pub enum MockError {
    NoMatchingId,
    PredicateError(PredicateError),
    Other(String),
}

impl From<String> for MockError {
    fn from(value: String) -> Self {
        MockError::Other(value)
    }
}
impl From<&str> for MockError {
    fn from(value: &str) -> Self {
        MockError::Other(value.into())
    }
}

//
#[test]
fn pointers1() {
    let a: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
        Box::new(|a| if *a > 2 { Ok(()) } else { Err("error".into()) });
    let double_ptr = ConditionDoublePointer::from_fn(a);
    let casted = unsafe { double_ptr.into_fn::<u32>() };
    assert!(casted(&3).is_ok());
    assert!(casted(&2).is_err());
}

#[test]
fn pointers2() {
    struct TestStruct {
        pub string: String,
    }

    let a: Box<dyn Fn() -> TestStruct + 'static> = Box::new(|| {
        println!("this is a closure return val");
        TestStruct {
            string: String::from("hello pointers2"),
        }
    });
    let double_ptr = ReturnValDoublePointer::from_fn(a);
    let casted = unsafe { double_ptr.into_fn::<TestStruct>() };
    assert_eq!(casted().string, "hello pointers2");
    assert_ne!(casted().string, "goodbye pointer2");
}

#[test]
fn context1() {
    println!("start of test");
    struct Foo(u32);
    struct Bar(String);
    let mock_id_foo = MockId("foo".into());
    let mock_id_bar = MockId("bar".into());
    let mut context_builder = ContextBuilder::new();
    assert!(
        context_builder
            .add_mock(mock_id_foo.clone(), Some(Box::new(|| Foo(42))))
            .is_ok()
    );
    let expectation1: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
        Box::new(|a| if *a == 7 { Ok(()) } else { Err("not 7".into()) });
    let expectation2 = |a: &u32| -> PredicateResult<()> {
        if *a == 42 {
            Ok(())
        } else {
            Err("not 42".into())
        }
    };
    let return_clos = || Foo(100);
    assert!(
        context_builder
            .add_expectation::<u32, Foo>(
                &mock_id_foo,
                expectation1,
                None,
                TimeModifier::Once,
                false
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation(
                &mock_id_foo,
                Box::new(expectation2),
                Some(Box::new(return_clos)),
                TimeModifier::Once,
                true
            )
            .is_ok()
    );

    let mut global_context = context_builder.finish();
    println!("here");
    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &7) else {
        panic!("failed first run");
    };
    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &result.0) else {
        panic!("failed first run");
    };
}

#[test]
fn context2() {
    println!("start of test");
    struct Foo(u32);
    struct Bar(String);
    let mock_id_foo = MockId("foo".into());
    let mock_id_bar = MockId("bar".into());
    let mut context_builder = ContextBuilder::new();
    assert!(
        context_builder
            .add_mock(mock_id_foo.clone(), Some(Box::new(|| Foo(42))))
            .is_ok()
    );
    assert!(
        context_builder
            .add_mock(mock_id_bar.clone(), Some(Box::new(|| Bar("getget".into()))))
            .is_ok()
    );

    let expectation1: Box<dyn Fn(&u32) -> PredicateResult<()> + 'static> =
        Box::new(|a| if *a == 7 { Ok(()) } else { Err("not 7".into()) });
    let expectation2 = |a: &u32| -> PredicateResult<()> {
        if *a == 42 {
            Ok(())
        } else {
            Err("not 42".into())
        }
    };
    let bar_expectation1: Box<dyn Fn(&Bar) -> PredicateResult<()> + 'static> = Box::new(|a| {
        if a.0 == "hello" {
            Ok(())
        } else {
            Err("not hello".into())
        }
    });
    let bar_expectation2 = |a: &Bar| -> PredicateResult<()> {
        if a.0 == "goodbye" {
            Ok(())
        } else {
            Err("bar not goodbye".into())
        }
    };
    let bar_ret1 = Box::new(|| Bar("goodbye".into()));

    let return_clos = || Foo(100);

    assert!(
        context_builder
            .add_expectation::<u32, Foo>(
                &mock_id_foo,
                expectation1,
                None,
                TimeModifier::Once,
                false
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation(
                &mock_id_foo,
                Box::new(expectation2),
                Some(Box::new(return_clos)),
                TimeModifier::Once,
                true
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation::<Bar, Bar>(
                &mock_id_bar,
                Box::new(bar_expectation1),
                Some(bar_ret1),
                TimeModifier::Once,
                false
            )
            .is_ok()
    );
    assert!(
        context_builder
            .add_expectation::<Bar, Bar>(
                &mock_id_bar,
                Box::new(bar_expectation2),
                None,
                TimeModifier::Once,
                true
            )
            .is_ok()
    );

    let mut global_context = context_builder.finish();
    println!("here");
    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &7) else {
        panic!("failed first run");
    };

    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), &result.0) else {
        panic!("failed second run");
    };

    let Ok(goodbye) =
        global_context.run_mock::<Bar, Bar>(mock_id_bar.clone(), &Bar("hello".into()))
    else {
        panic!("failed third run");
    };

    let Ok(result) = global_context.run_mock::<Bar, Bar>(mock_id_bar.clone(), &goodbye) else {
        panic!("failed fourth run");
    };
}
