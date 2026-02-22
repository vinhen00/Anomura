use api_macro::make_answer;

#[test]
pub fn mac_test() {
    make_answer!();
    assert_eq!(answer(), 42);
}
