use example::number_server::{local_test, remote_test};

fn main() {
    let local = true;
    if local {
        local_test();
    } else {
        remote_test();
    }
}
