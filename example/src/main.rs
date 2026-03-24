use rrmi_macros::remote_object;
struct Calculator;

#[remote_object]
impl Calculator {
    #[remote]
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    #[remote]
    fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
    fn sub(&self, a: i32, b: i32) -> i32 {
        a - b
    }
}

fn main() {
    let c = Calculator;
    let (a, b) = (1, 2);
    c.add(a, b);
    c.multiply(a, b);
    c.sub(a, b);
}
