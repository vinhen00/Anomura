

Ok so, add some changes to how the system for expectations are handled.
our mock objects for ADTS have the following structure:

MockObjects MockIds are the path of the object + adt_mock_id
MockObject {
    ..<for all methods tied to the object, create a MockFunctionObject>
}

MockFunctionObject {
    a list of expectations,
    a list of on_call
}

on_call also takes a boolean predicate with the function input similarly to expects. the only difference is that we don't expect that a on_call will be fulfilled. so exhaustion and completeness is not needed. we simply check if the condition is met and use the defined return value if that is the case. A mockFunctionObject can have multiple onCalls.

checkpoints contain a separate list for standalone functions that are mockfunctionobjects aswell.

constructor methods are special in that the user defines a closure that returns Public<Structname> rather than <Structname>, in order not to expose the user to the structs private fields.