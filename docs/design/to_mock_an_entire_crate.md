


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
