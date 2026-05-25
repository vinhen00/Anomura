use mock_macro::mock_fn;
#[test]
fn macro_test1() {
    mock_fn!(
        rand,
        fn random_bool(prob: f64) -> bool {
            default_return({ true });
            expect(
                *prob > 0.5,
                once(),
                with_return({
                    println!("set return value false for first expect");
                    false
                }),
            );
            expect(*prob < 0.9, once());
        }
    );
    context::finish_building_context();
    let fst = rand::random_bool(0.7);
    println!("fst return val {fst}");
    if !fst {
        let snd = rand::random_bool(0.89);
        println!("second return val {snd}");
    }
}
