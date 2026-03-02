use std::collections::HashMap;
use std::any::Any;




pub struct GlobalContext {
    mocks: HashMap<String, Box<dyn Any>>
}

impl GlobalContext{
    pub fn new() -> Self {
        GlobalContext { mocks: HashMap::new() }
    }

    pub fn get_mock(&mut self, name: String) -> Option<&mut Box<dyn Any>> {
        self.mocks.get_mut(&name)
    }

    pub fn insert_mock(&mut self,name: String , mock: Box<dyn Any>) {
        self.mocks.insert(name,mock);
    }
}

pub struct MockFunction<Args> {
    times_called: i32,
    call_list: Vec<(i32, Args)>
}

impl<Args: Clone> MockFunction<Args> {
    pub fn new() -> Self {
        MockFunction { times_called: 0, call_list: Vec::new() }
    }

    // pub fn unbox(input: Option<Box<MockFunction<Args>>>) -> Option<Self> {
    //     if let Some(boxed) = input {
    //         if let Some(stats) = boxed.downcast_mut::<MockFunction<(i32, String, i32)>>(){
    //                 Some(stats)               
    //             }
    //         }
    //         else {None}
    //     }
    //     else {None}
    // }

    pub fn get_count(&self) -> i32 {
        self.times_called
    }

    pub fn get_call_list(&self) -> Vec<(i32, Args)> {
        self.call_list.clone()
    }

    pub fn incr_count(&mut self) {
        self.times_called += 1;
    }

    pub fn add_call(&mut self, arg: Args) {
        let new_call = (self.times_called, arg);
        self.call_list.push(new_call);
    }
}