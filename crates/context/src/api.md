




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
