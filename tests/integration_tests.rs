extern crate vinwolf;

use vinwolf::prueba::add;

mod safrole;


#[test]
fn lo_prueba() {
    assert_eq!(4, add(2, 2));
}