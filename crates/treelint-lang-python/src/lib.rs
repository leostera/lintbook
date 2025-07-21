pub mod py001_no_try_catch;
pub mod py002_no_sys_path_modification;
pub mod py003_no_os_getenv;
pub mod py004_no_bare_except;
pub mod py005_none_comparison;
pub mod py006_true_false_comparison;
// pub mod py007_not_in_test;
// pub mod py008_not_is_test;
// pub mod py009_type_comparison;

use treelint_core::Rule;

/// Returns all Python lint rules
pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(py001_no_try_catch::NoTryCatch),
        Box::new(py002_no_sys_path_modification::NoSysPathModification),
        Box::new(py003_no_os_getenv::NoOsGetenv),
        Box::new(py004_no_bare_except::NoBareExcept),
        Box::new(py005_none_comparison::NoneComparison),
        Box::new(py006_true_false_comparison::TrueFalseComparison),
        // Box::new(py007_not_in_test::NotInTest),
        // Box::new(py008_not_is_test::NotIsTest),
        // Box::new(py009_type_comparison::TypeComparison),
    ]
}
