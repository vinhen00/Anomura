use std::{
    collections::HashMap,
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

pub static GLOBAL_CONTEXT: OnceLock<Mutex<GlobalContext>> = OnceLock::new();
pub fn context() {
    println!("context says hello");
}

pub enum SequenceState {
    Inactive,
    Active,
}
pub struct SequenceHead {
    effected_mocks: Vec<MockId>,
    sequence_state: SequenceState,
    index: u32,
}
pub enum MockState {
    Locked { sequence_head_index: NodeIndex<u32> },
    Unlocked { mock_head_index: NodeIndex<u32> },
}
pub struct MockHead {
    state: MockState,
    default_return_val: Option<ReturnValDoublePointer>,
}

pub const CONTEXT_CONST: &str = "CONSTANT FROM CONTEXT";
#[derive(Debug, Clone, Hash, FromStr, AsRef, AsMut, core::cmp::Eq, PartialEq)]
pub struct MockId(String);
pub struct GlobalContext {
    sequence_heads: Vec<SequenceHead>,

    graph: DiGraph<MockNode, Edge>,

    mock_heads: HashMap<MockId, MockHead>,
}

impl GlobalContext {
    pub fn get_dot(&self) -> String {
        let dot = petgraph::dot::Dot::new(&self.graph);
        format!("{dot:?}")
    }

    pub fn run_mock<Input, ReturnVal>(
        &mut self,
        mock_id: MockId,
        input: Input,
    ) -> Result<ReturnVal> {
        let Some(head) = self.mock_heads.get_mut(&mock_id) else {
            return Err(format!("couldn't find mock {:?} in context", mock_id).into());
        };
        let index = match head.state {
            MockState::Locked { .. } => todo!(),
            MockState::Unlocked { mock_head_index } => mock_head_index,
        };
        let Some(res) = self.graph.node_weight(index) else {
            return Err("node not found".into());
        };
        if res.id != mock_id {
            return Err(format!(
                "node with index {:?} expected {:?} but received {:?}",
                index, mock_id, res.id
            )
            .into());
        };
        let mut edges: Vec<_> = self
            .graph
            .edges_directed(index, petgraph::Direction::Outgoing)
            .filter_map(|e| match e.weight() {
                Edge::Instant { priority } => Some((*priority, None, e.target())),
                Edge::Condition(conditional_edge) => {
                    let condition = unsafe {
                        conditional_edge
                            .condition
                            .into_fn::<Input>()
                            .expect("failed to dereference function pointer")
                    };
                    let res = condition(&input);
                    if res.is_ok() {
                        Some((
                            conditional_edge.priority,
                            conditional_edge.return_val.clone(),
                            e.target(),
                        ))
                    } else {
                        None
                    }
                }
            })
            .collect();
        if edges.is_empty() {
            return Err("test failed, no valid conditions for input".into());
        };
        edges.sort_by(|e, f| e.0.cmp(&f.0));
        let edge = edges
            .last()
            .expect("we have already checked that edges is not empty");

        let return_val_ptr = edge.1.as_ref().unwrap_or(
            head.default_return_val
                .as_ref()
                .expect("no return value found"),
        );
        let return_closure = unsafe {
            return_val_ptr
                .into_fn::<ReturnVal>()
                .expect("failed to turn return value ptr to ref")
        };
        let return_val = return_closure();

        let new_node_index = edge.2;
        let Some(new_node) = self.graph.node_weight(new_node_index) else {
            return Err(format!("new node index {:?} is invalid", new_node_index).into());
        };
        if new_node.id != mock_id {
            return Err(format!(
                "new node with index {:?} expected mockid {:?}, but got {:?}",
                new_node_index, mock_id, new_node.id
            )
            .into());
        }
        head.state = MockState::Locked {
            sequence_head_index: new_node_index,
        };
        Ok(return_val)
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
pub struct ConditionDoublePointer {
    thin_ptr: usize,
}
#[derive(Debug, Clone)]
pub struct ReturnValDoublePointer {
    thin_ptr: usize,
}
impl ReturnValDoublePointer {
    pub fn from_fn<ReturnVal>(closure: Box<dyn Fn() -> ReturnVal>) -> Self {
        let thick_ptr = Box::leak(closure);
        let thin_ptr = (&thick_ptr) as *const _;
        Self {
            thin_ptr: thin_ptr as usize,
        }
    }
    pub unsafe fn into_fn<ReturnVal>(&self) -> Result<&dyn Fn() -> ReturnVal> {
        let thick_ptr: *mut dyn Fn() -> ReturnVal = unsafe { *(self.thin_ptr as *const _) };
        let ptr_ref: &dyn Fn() -> ReturnVal =
            unsafe { thick_ptr.as_ref().expect("failed to cast thick_ptr to ref") };
        Ok(ptr_ref)
    }
}
impl ConditionDoublePointer {
    pub fn from_fn<Input>(closure: Box<dyn Fn(&Input) -> Result<()> + 'static>) -> Self {
        let thick_ptr = Box::leak(closure);
        let thin_ptr = (&thick_ptr) as *const _;
        Self {
            thin_ptr: thin_ptr as usize,
        }
    }
    pub unsafe fn into_fn<Input>(&self) -> Result<&dyn Fn(&Input) -> Result<()>> {
        let thick_ptr: *mut dyn Fn(&Input) -> Result<()> = unsafe { *(self.thin_ptr as *const _) };
        let ptr_ref: &dyn Fn(&Input) -> Result<()> =
            unsafe { thick_ptr.as_ref().expect("failed to cast thick_ptr to ref") };
        Ok(ptr_ref)
    }
}

pub struct ContextBuilder {
    mocks: HashMap<MockId, MockBuilder>,
    graph: DiGraph<MockNode, Edge>,
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
        condition: Box<dyn Fn(&Input) -> Result<()>>,
        return_val_closure: Option<Box<dyn Fn() -> ReturnVal>>,
        modifier: TimeModifier,
        exit: bool,
    ) -> Result<()> {
        let return_val_as_ptr =
            return_val_closure.map(|closure| ReturnValDoublePointer::from_fn(closure));
        let condition_as_ptr = ConditionDoublePointer::from_fn(condition);
        let Some(builder) = self.mocks.get_mut(&mock_id) else {
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
                //add two edges to new node, one with always ( lowest priority, one with condition)
                //      epsilon || condition
                // (1) ----------------------> (2)
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    condition: condition_as_ptr.clone(),
                    return_val: return_val_as_ptr.clone(),
                });
                let instant_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    condition: condition_as_ptr,
                    return_val: return_val_as_ptr,
                });
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
                //       /   \
                //      |    |
                //       \  /         epsilon
                //        (1) ------------------> (2)
                let main_weight = Edge::Condition(ConditionalEdge {
                    priority: 1,
                    condition: condition_as_ptr.clone(),
                    return_val: return_val_as_ptr.clone(),
                });
                let instant_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    condition: condition_as_ptr,
                    return_val: return_val_as_ptr,
                });
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

                let instant_weight = Edge::Condition(ConditionalEdge {
                    priority: 0,
                    condition: condition_as_ptr,
                    return_val: return_val_as_ptr,
                });
                //once
                self.graph
                    .add_edge(builder.head, n_plus_one, main_weight.clone());

                //consume input edge
                self.graph.add_edge(n_plus_one, n_plus_one, main_weight);
                //epsilon edge
                self.graph
                    .add_edge(n_plus_one, new_node_index, instant_weight);
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

pub enum TimesValue {
    Explicit(u32),
    Implicit(EnvId),
}
pub enum TimeModifier {
    Once,
    AtMostOnce,
    Any,
    AtLeastOnce,
    Until(EnvId),
    Times(TimesValue),
    After(EnvId),
}

#[derive(Debug, Clone)]
pub struct ConditionalEdge {
    priority: u8,
    condition: ConditionDoublePointer,
    return_val: Option<ReturnValDoublePointer>,
}

impl ConditionalEdge {}

#[derive(Debug, Clone)]
pub enum Edge {
    Instant { priority: u8 },
    Condition(ConditionalEdge),
}
#[derive(Debug, Clone)]
pub struct MockNode {
    entry: bool,
    exit: bool,
    id: MockId,
}

pub type Result<T> = result::Result<T, MockError>;

#[derive(Debug, Clone, Display, FromStr)]
pub struct MockError(String);

impl From<String> for MockError {
    fn from(value: String) -> Self {
        MockError(value)
    }
}
impl From<&str> for MockError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

//
#[test]
fn pointers1() {
    let a: Box<dyn Fn(&(i32, &i32)) -> Result<()> + 'static> = Box::new(|(a, b): &(i32, &i32)| {
        if *a > 2 && **b > 5 {
            Ok(())
        } else {
            Err("error".into())
        }
    });
    let double_ptr = ConditionDoublePointer::from_fn(a);
    match unsafe { double_ptr.into_fn::<(i32, &i32)>() } {
        Ok(casted) => {
            assert!((*casted)(&(3, &6)).is_ok());
            assert!((*casted)(&(2, &6)).is_err());
        }
        Err(e) => panic!("failed to cast pointer with error : {:?}", e),
    }
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
    match unsafe { double_ptr.into_fn::<TestStruct>() } {
        Ok(casted) => {
            assert_eq!((*casted)().string, "hello pointers2");
            assert_ne!((*casted)().string, "goodbye pointer2");
        }
        Err(e) => panic!("failed to cast pointer with error : {:?}", e),
    }
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
    let expectation1 =
        |a: &u32| -> Result<()> { if *a == 7 { Ok(()) } else { Err("not 7".into()) } };
    let expectation2 = |a: &u32| -> Result<()> {
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
                Box::new(expectation1),
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
    let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), 7) else {
        panic!("failed first run");
    };
    /*let Ok(result) = global_context.run_mock::<u32, Foo>(mock_id_foo.clone(), result.0) else {
        panic!("failed second run");
    };*/
}
