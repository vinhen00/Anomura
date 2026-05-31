use bar::{Bar};
use mock_macro::{mock_fn, mock_method, mock_struct};
use std::sync::Mutex;
fn main() {
}





#[test]
fn struct_example() {
    mock_struct!(
        bar,
        struct Bar{
            pub new_field: u32
        }
        fn new() -> Bar {
            default_return(
                Bar{
                field1: 1,
                field2: 3,
                new_field: 4
            });
        }
        []
    );

    mock_method!(
        bar,
        Bar,
        fn method1(&self) {
            default_return(());
            expect(self.new_field == 4, once());
        }
    );

    mock_method!(
        bar,
        Bar,
        fn method2(&self) {
            default_return(());
            expect(self.mock_hash == 1.to_string(), once());
        }
    );
    context::finish_building_context();

    let obj1 = bar::Bar::new();
    let obj2 = bar::Bar::new();

    obj1.method1();
    obj2.method2();


}
