use bench::benches;
use criterion::Criterion;
use rb_sys_test_helpers::setup_ruby;

fn main() {
    let _cleanup = unsafe { setup_ruby() };
    benches();
    Criterion::default().configure_from_args().final_summary();
}
