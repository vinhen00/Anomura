


# overhaul of crate mocking

## Basic reasoning and motivation
there is no way to to mock a single struct or trait removing the private implementation while being able to make sure that the removal remains controlled and unpropagated to the rest of the module structure. 
A crate is the smallest software module that where we can reliably keep outfacing APIs while completetly removing/replacing implementation.




## Private traits
private traits could probably be emptied of methods but must still exist.
## public traits
public traits keep all of their public endpoints.
they are extended by the mockable trait by default
all public methods be mocked by default? 
## How do we treat ADTs (Structs, Enums, Unions)?
All private fields in ADTs should be replaced with phantom data versions of their types. 
All public fields must stay, additional fields could be added. 
All traits must still be implemented. 


regarding Ids, how do we keep track of instances?

If we are to mock all the ADTs in the crate. There are some problematic cases that will not allow us to differentiate instances by adding an id as a field without modifying the api. 
structs with all public fields.
enum variants

one thought was to use the unstable allocator_api to use the address of an object instead of its id. however, this would only work for heap allocated mock objects, there is no way to track stack allocated instances through their addresses. The same goes for zero-sized structs.

If we were to stick with using private ids added to the structs, then we cannot mock structs with public fields or literals, or there can at least not be a way to differentiate instances. We would otherwise be breaking the public Api. Same thing goes for Enums with variants that don't wrap a single definition that also does not contain all public fields. How do we deal with these? for the sake of testing implementation we simply don't differentiate instances for them.
zero-sized structs should be fine, adding a private field should not change the api.



the id added to the ADT is not public and the user should not be able to initialize it. 
All constructor methods needs to implement logic to sync the instance with the context

#### how should ADTs use drop?
drop should be implemented to remove ADT from context, and check if all predicates for it had been met, the user can decide if this should be a hard failure for the test or not.
drop references the id in the context, checks all expectation and makes sure that they have passed, and then uses the special id_drop() method on them to clean them up.

## public standalone functions
mock objects for these are added automatically
#### quick note on polymorphic standalone functions
we can fake post-monomorphized separation between expectations by dynamically checking type-ids. Let the user define an expectation for a call with a specific type, then in supplanted function body check for the defined type and let them be connected to different mock ids.
## private standalone functions are removed


## background for the uninitiated: what are we putting in all the methods?
i'll use method to refer to both methods and functions unless otherwise stated.
when mocking a crate, it is linked to a context crate that manages all expectations tied to mocks. 
in the methods, we create a mock_id that corresponds to the method path + an id tied to the mocked ADT. this id is also mirrored in the global context object. When an ADT is initialized, a mock object corresponding to this id is added to the context. 
expectations and 
When a method is called, It tries to match its id against the context and the expectations that have been added to it.


notes from author to author: how do i keep track of ids and signature differences between methods for the same object
each expectation put on a method not only ties to an id. but a method id.
ADT id: path to ADT + unique id per instance. 
method id: path to method 








### injecting extra traits (default)
In most cases (unless there is a negative trait bound for example) there should not be issues caused by adding simple quality of life traits such as default. And in case of such a conflict, the error will be caught at compile time. 


## Benefits in terms of mocking api vs current solution. 
We would not have to rely on macros to generate mocked ADTs (structs, enums, unions) instances. 
It should be possible to let all ADTs implement a new trait "Mockable" or something similar, that allows for interaction with the global context, instanciation. 
!!Problem, linting! We need to figure out a way to deal with the linting issue, since these traits won't exist in the mocked crates original project file. The code would still compile, but it would be very ugly and unergonomic. One strategy suggested by David was to create a "ghost-project", a decompiled version of the mocked crates modified AST that Rust-analyzer could be redirected to instead of the actual crate.

What we would need to fix to make this a reality: The span of our modified file is corrupted after our rewrites, this is probably very preventable and possible to fix.
Syn is able to turn Rust Ast code into text very easily. We already utilize conversions into text in the mocking process, partially through syn. could we recursively use quote and similar methods for our entire project ast? I think this is very likely possible.

we could perhaps more easily integrate with google test match dsl 


### creating a minimal proof of concept




```rust
    mock_crate!(krate);
    //mock object is created
    let example_mock = Example::new(2.0,1.0);
    example_mock.on_call_meth1(|&self, a: &f32, b: &f32|)
    example_mock.expect_meth1(
        |slf: &Example, a: &f32, b: &f32| {a >= self.a, b >= self.b },
        |&self, a: f32, b: f32|           {a >= self.a, b >= self.b },
        None
        )


´´´


all structs will implement their own unique traits mock<structname>

```rust
    // in crate named krate
    mod Mod {
        pub Struct Example {
            a : f32,
            b : pub f32
        }

        pub trait ExTrait {
            pub fn meth2(&mut self, text : String) -> bool {

            } 
        }
        impl ExTrait for Example {
            ..
        }
        //original impl
        impl Example {
            fn meth1(&self, a: f32, b : f32) -> usize;
            fn new(a: f32, b: f32) -> Self;
        }
        impl From<(f32,f32)> for Example {
            ...
        }

        should generate something like
        // Example is given an id field, all private fields are made into PhantomData
        pub Struct Example {
            a : PhantomData<f32>,
            b : pub f32,
            adt_mock_id : context::MockId, 
        }



        impl Mockable for Example {
            /// can only be called once
            fn add_on_call<Return>(expr : Fn);

            fn create_expectation<Input, Ret>(expr : Fn<Input> -> Result<()>) -> Expectation<Input,Ret>;
            fn add_expectation<Input,Ret>(expr : Expectation<Input>, ret : Option<Ret>, time_modifier : Option<TimeModifier>);
        }

        impl Drop for Example {
            fn drop(self) {
                //lookup context, find all the saved expectations, cast to closures using the right ids and drop them
            }
        }

        pub struct PredicateExampleMeth1(context::Predicate);
        pub struct ExpectationExampleMeth1(context::Expectation);
        pub struct ReturnExampleMeth1(context::ReturnValPointer);
        pub struct PredicateExampleNew;
        pub struct ExpectationExampleNew;
        pub struct ReturnExampleNew;
        //trait impls    
        pub struct PredicateExampleInto(context::Predicate);
        pub struct ExpectationExampleInto(context::Expectation);
        pub struct ReturnExampleInto(context::ReturnValPointer);
        
        pub struct PredicateExampleImplExTraitMeth2(context::Predicate);
        pub struct ExpectationExampleImplExTraitMeth2(context::Expectation);
        pub struct ReturnExampleImplExTraitMeth2(context::ReturnValPointer);

        impl PredicateExampleMeth1 {
            pub fn new(F : impl Fn<&Example, &f32, &f32>) -> bool {
                context::add_expectation(MockId::new("PredicateExampleMeth1"))
            }
        }
        impl <F : impl Fn<&Example, &f32,&f32> -> bool> From<F> for PredicateExampleMeth1 {
            ..  
        }
        impl <F : impl Fn<Example, f32,f32> -> usize> From<F> for ReturnExampleMeth1 {
            ..   
        }
        impl From<PredicateExampleMeth1> for ExpectationExampleMeth1 {
            ..
        }

        impl Example {
            ... added for all methods

            pub fn meth1(&self, a : f32, b : f32) -> usize {
                ..
            }
            pub fn on_call_meth1(ret : impl Into<ReturnExampleInto>) {
                ..
            }
            pub fn create_predicate_for_meth1() -> Predicate<(&Self,f32,f32), usize> {
                ..
            }

            pub fn expect_meth1(expect : impl Into<ExpectationExampleMeth1>, Option<impl Into<ReturnExampleMeth1>>, time_mod : Option<TimeModifier>) -> ret {
                ..
            }



            pub fn new(a :f32, b : f32) -> Self {
                std::println!("Mocked version of method {} was used", #name_str);
                //since new is a constructor we must first initialize the mock object
                let slf = Self {
                    a : f32,
                    b : PhantomData::new(),
                    adt_mock_id : context::new_id()

                    }
                context::add_mock<(a: f32, b: f32), Self>()
                if context::ctx_built_and_contains_id(&#mock_id_ident) {
                    match context::run_mock::<(f32,f32), Self>(#mock_id_ident, #input_ident_tuple) {
                        Ok(res) => res,
                        Err(e) => match e {
                            context::MockError::Other(e) => panic!("unexpected Error: {:?}", e),
                            context::MockError::PredicateError(e) => panic!("{:?}", e.0),
                            context::MockError::NoMatchingId => panic!("failed to find mock id"),
                        }
                    }
                }
              
        }
                
            }        
            pub fn on_call_new(ret : impl Fn<f32,f32> -> usize);
            
            pub fn create_predicate_for_new(expectation : impl Into<PredicateExampleNew>) -> PredicateExampleNew {
                // initialize mock object
            }

            pub fn expect_new(expectation, ret, time_mod : Option<TimeModifier>) -> ret {
                ..
            }
        
    }

´´´