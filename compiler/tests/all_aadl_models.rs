use compiler::test_mod;
use compiler::test_mod2;

#[test]
fn all_aadl_models_should_generate_rust_code() {
    //test_mod::run_all_test_cases();
    test_mod2::run_all_case_folders();
}
