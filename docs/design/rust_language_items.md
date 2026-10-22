# Rust Language Items

- Modules (`mod`)
    -kept as is
- Use Declarations (`use`)
    -kept as is
- Extern Crate Declarations (`extern crate`)
- Structs (`struct`)
- Enumerations (`enum`)
- Unions (`union`)
- Type Aliases (`type`)
- Functions (`fn`)
    * private
        private functions are removed
    * public
        body removed and replaced with mocked function, helper structs and functions are added

- Traits (`trait`)
    * private
        kept but all contained methods are removed
    * public
        - kept, all methods are mockable. 
        - IDEA: ADTs that implement a trait will have helper functions and methods generated to mock them. however, we should be able to set expectations and on_call on traits directly, so that they can be called when in the form of trait objects aswell (if compatible)
        we probably don't need to keep the concrete implementation of a function, could this cause confusion? it would be nice if we could keep identities regardless if the trait version or the concrete version of a type.
        
        how do trait methods interact with different types of mock ADTs?
            - instanced: full functionality
            - non-instanced: may not be possible to track ADT version when represented as trait object.
                possible solution: let the non-instanced struct implement a new trait "non-instanced-mock", with a single method that returns a signifier for the ADT.
                let there be an generic implementation for the trait for trait : non_instanced_mock. that picks utilizes the id.
- Implementations (`impl`)
- Constants (`const`)
    Keep Public as is. remove private ones

- Statics (`static`)
    Keep as is? remove private ones
- Declarative Macros (`macro_rules!`)
    this is tricky af...
- Procedural Macros
    this is also tricky af
- Extern Blocks (`extern`)

- Global Assembly (`global_asm!`)
    urghh idk