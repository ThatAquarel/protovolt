mod support;

use support::TestBench;

#[test]
fn test_bench_exec_idn() {
    let mut bench = TestBench::standby();
    let resp = bench.exec("*IDN?").unwrap();
    assert!(!resp.is_empty());
}

#[test]
fn test_bench_last_error_empty() {
    let mut bench = TestBench::standby();
    let (code, msg) = bench.last_error();
    assert_eq!(code, 0);
    assert_eq!(msg.as_str(), "No error");
}
