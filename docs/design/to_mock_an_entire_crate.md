


# overhaul of crate mocking

## Basic reasoning and motivation
there is no way to to mock a single struct or trait removing the private implementation while being able to make sure that the removal remains controlled and unpropagated to the rest of the module structure. 
A crate is the smallest software module that where we can reliably keep outfacing APIs while completetly removing/replacing implementation.


# Private traits
private traits could probably be emptied of methods but must still exist.
# public traits
public traits keep all of their public endpoints.
should default methods be mocked?
should all public methods be mocked by default? i don't see why not. 
## How do we treat ADTs?
All private fields in ADTs should be replaced with phantom data versions of their types. 
All public fields must stay, additional fields could be added. 
All traits must still be implemented. 

### injecting extra traits (default)
In most cases (unless there is a negative trait bound for example) there should not be issues caused by adding simple quality of life traits such as default. And in case of such a conflict, the error will be caught at compile time. 

### Do we need a special case for constructors? (i.e Into, new, default)


## Benefits in terms of mocking api. 
We would not have to rely on macros to generate mocked ADTs (structs, enums, unions) instances. 
It should be possible to let all ADTs implement a new trait "Mockable" or something similar, that allows for interaction with the global context, instanciation. 
!!Problem, linting! We need to figure out a way to deal with the linting issue, since these traits won't exist in the mocked crates original project file. The code would still compile, but it would be very ugly and unergonomic. One strategy suggested by David was to create a "ghost-project", a decompiled version of the mocked crates modified AST that Rust-analyzer could be redirected to instead of the actual crate.

What we would need to fix to make this a reality: The span of our modified file is corrupted after our rewrites, this is probably very preventable and possible to fix.
Syn is able to turn Rust Ast code into text very easily and we conversions into text in the mocking process, partially through syn. could we recursively use quote and similar methods for our entire project ast? I think this is very likely possible.
