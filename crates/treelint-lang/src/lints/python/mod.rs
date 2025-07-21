pub mod py001_no_try_catch;
pub mod py002_no_sys_path_modification;

use crate::lints::Rule;

/// Returns all Python lint rules
pub fn get_python_lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(py001_no_try_catch::NoTryCatch),
        Box::new(py002_no_sys_path_modification::NoSysPathModification),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_python_lints_returns_rules() {
        let lints = get_python_lints();
        assert_eq!(lints.len(), 2);

        // Check first lint
        assert_eq!(lints[0].id(), "PY001");
        assert_eq!(lints[0].name(), "no-try-catch");
        assert_eq!(lints[0].description(), "Disallow try/except statements");

        // Check second lint
        assert_eq!(lints[1].id(), "PY002");
        assert_eq!(lints[1].name(), "no-sys-path-modification");
        assert_eq!(lints[1].description(), "Disallow modification of sys.path");
    }
}
