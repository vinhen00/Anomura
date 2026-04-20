




mock!(list of valid mock statements (Stm))

time modifier can be added to expectation, or expectation chain


Stm := mock_fn!(path, fn <name> (..<args>) -> -<return-type> {
    default_return_val();

    expect(<args>).once();
    expect(closure).at_most_once();
    (
     expect(..);
     expect(..).return(..);   
    ).any()
});




any for seq of size of at least 1: 
    start     end
    a -> b -> c
    0 -epsilon->1
    merge start 0, merge end with 0
    0 -epsilon->1, also 0 -> b -> 0
once for seq of size of at least 1:
    simple append
    0 -> a -> *-> c
at most once
    0 -> a -> b -> c
at least once
    0 -> a -> b -> c -> 02 -> b -> ->02 -epsilon -> 1

at most once
    0 -epsilon->1
    0-cond -> 1


let sequence = NewSequence(exp2, exp3)


add(exp0); add(exp1);
let x: SeqSlice = (exp1.any, exp2.once,).any; 
x.follows(exp0);
(if exp3 {x} else {exp0})


let x = #[mock(path = ..)] Struct Foo {

}

initialize_x()
