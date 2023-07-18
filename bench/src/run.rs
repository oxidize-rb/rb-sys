use bench::benches;
use criterion::Criterion;
use rb_sys_test_helpers::setup_ruby_unguarded;

fn main() {
    let criterion = Criterion::default().configure_from_args();
    unsafe { setup_ruby_unguarded() };
    benches();
    criterion.final_summary();
}
