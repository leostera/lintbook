use tree_sitter::{Node, Tree};
use treelint_core::*;

pub struct ApproxConstant;

impl Rule for ApproxConstant {
    fn id(&self) -> &'static str {
        "RS003"
    }

    fn name(&self) -> &'static str {
        "approx-constant"
    }

    fn description(&self) -> &'static str {
        "Checks for approximate float constants that should use mathematical constants"
    }

    fn explanation(&self) -> &'static str {
        "Using hardcoded approximate values instead of mathematical constants reduces readability \
         and precision. Consider using constants like std::f64::consts::PI instead of 3.14159."
    }

    fn check(&self, tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        self.check_node(tree.root_node(), source, &mut violations);
        violations
    }
}

impl ApproxConstant {
    fn check_node(&self, node: Node, source: &str, violations: &mut Vec<LintViolation>) {
        if node.kind() == "float_literal" {
            let text = &source[node.byte_range()];
            if let Some(constant_name) = check_approximate_constant(text) {
                let position = node.start_position();
                violations.push(LintViolation {
                    line: position.row + 1,
                    column: position.column + 1,
                    message: format!(
                        "Approximate constant `{}` found, consider using `{}`",
                        text, constant_name
                    ),
                    lint_name: self.name().to_string(),
                    lint_id: self.id().to_string(),
                });
            }
        }

        // Recursively check child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(child, source, violations);
        }
    }
}

fn check_approximate_constant(text: &str) -> Option<&'static str> {
    // Remove common suffixes
    let cleaned = text.trim_end_matches("f32").trim_end_matches("f64");
    
    if let Ok(value) = cleaned.parse::<f64>() {
        // Check against known mathematical constants with some tolerance
        const TOLERANCE: f64 = 0.001;
        
        // π (pi) ≈ 3.14159265359
        if (value - std::f64::consts::PI).abs() < TOLERANCE {
            return Some("std::f64::consts::PI");
        }
        
        // e ≈ 2.71828182846
        if (value - std::f64::consts::E).abs() < TOLERANCE {
            return Some("std::f64::consts::E");
        }
        
        // √2 ≈ 1.41421356237
        if (value - std::f64::consts::SQRT_2).abs() < TOLERANCE {
            return Some("std::f64::consts::SQRT_2");
        }
        
        // ln(2) ≈ 0.69314718056
        if (value - std::f64::consts::LN_2).abs() < TOLERANCE {
            return Some("std::f64::consts::LN_2");
        }
        
        // ln(10) ≈ 2.30258509299
        if (value - std::f64::consts::LN_10).abs() < TOLERANCE {
            return Some("std::f64::consts::LN_10");
        }
        
        // log₂(e) ≈ 1.44269504089
        if (value - std::f64::consts::LOG2_E).abs() < TOLERANCE {
            return Some("std::f64::consts::LOG2_E");
        }
        
        // log₁₀(e) ≈ 0.43429448190
        if (value - std::f64::consts::LOG10_E).abs() < TOLERANCE {
            return Some("std::f64::consts::LOG10_E");
        }
        
        // 1/π ≈ 0.31830988618
        if (value - std::f64::consts::FRAC_1_PI).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_1_PI");
        }
        
        // 2/π ≈ 0.63661977236
        if (value - std::f64::consts::FRAC_2_PI).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_2_PI");
        }
        
        // π/2 ≈ 1.57079632679
        if (value - std::f64::consts::FRAC_PI_2).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_PI_2");
        }
        
        // π/3 ≈ 1.04719755120
        if (value - std::f64::consts::FRAC_PI_3).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_PI_3");
        }
        
        // π/4 ≈ 0.78539816340
        if (value - std::f64::consts::FRAC_PI_4).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_PI_4");
        }
        
        // π/6 ≈ 0.52359877560
        if (value - std::f64::consts::FRAC_PI_6).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_PI_6");
        }
        
        // π/8 ≈ 0.39269908170
        if (value - std::f64::consts::FRAC_PI_8).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_PI_8");
        }
        
        // 1/√2 ≈ 0.70710678118
        if (value - std::f64::consts::FRAC_1_SQRT_2).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_1_SQRT_2");
        }
        
        // 2/√π ≈ 1.12837916710
        if (value - std::f64::consts::FRAC_2_SQRT_PI).abs() < TOLERANCE {
            return Some("std::f64::consts::FRAC_2_SQRT_PI");
        }
    }
    
    None
}