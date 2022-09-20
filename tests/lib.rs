use ibm_5150::*;

#[test]
fn test_main() {
    let mut sys = IbmPc::new();

    sys.run_test();
}
