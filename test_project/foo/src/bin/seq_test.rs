use mock_macro::mock_method;
use bar::{red,blue};

context::new_sequence("seq1");
mock_fn!(
    foo,
    fn blue(input: i32) {
        expect(..);
        enter_seq(seq1);
        expect_seq(.., seq1,Index(0));
        expect(..);
    });
mock_fn!(
    foo,
    fn red(input: i32) {
        expect(..);
        expect(..);
        enter_seq(seq1);
        expect_seq(.., seq1, Index(1));
        expect(..);
    });