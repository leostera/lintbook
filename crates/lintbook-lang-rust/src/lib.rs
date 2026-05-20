pub mod rs001_absurd_extreme_comparisons;
pub mod rs002_almost_swapped;
pub mod rs003_approx_constant;
pub mod rs004_async_yields_async;
pub mod rs013_eq_op;
pub mod rs014_erasing_op;
pub mod rs016_ifs_same_cond;
pub mod rs022_inline_fn_without_body;
pub mod rs025_invisible_characters;
pub mod rs026_iter_next_loop;
pub mod rs027_iter_skip_zero;
pub mod rs028_iterator_step_by_zero;
pub mod rs029_let_underscore_lock;
pub mod rs032_mem_replace_with_uninit;
pub mod rs034_mistyped_literal_suffixes;
pub mod rs035_modulo_one;
pub mod rs038_non_octal_unix_permissions;
pub mod rs041_option_env_unwrap;
pub mod rs046_possible_missing_comma;
pub mod rs050_reversed_empty_ranges;
pub mod rs051_self_assignment;
pub mod rs054_suspicious_splitn;
pub mod rs055_transmute_null_to_fn;
pub mod rs056_transmuting_null;
pub mod rs057_uninit_assumed_init;
pub mod rs058_uninit_vec;
pub mod rs059_unit_cmp;
pub mod rs060_unit_hash;
pub mod rs063_unused_io_amount;
pub mod rs064_useless_attribute;
pub mod rs065_vec_resize_to_zero;
pub mod rs075_cast_abs_to_unsigned;
pub mod rs079_cast_slice_from_raw_parts;
pub mod rs081_const_is_empty;
pub mod rs082_crate_in_macro_def;
pub mod rs083_deprecated_clippy_cfg_attr;
pub mod rs087_duplicate_mod;
pub mod rs088_duplicated_attributes;
pub mod rs089_empty_docs;
pub mod rs090_empty_line_after_doc_comments;
pub mod rs091_empty_line_after_outer_attr;
pub mod rs092_empty_loop;
pub mod rs095_four_forward_slashes;
pub mod rs096_from_raw_with_void_ptr;
pub mod rs101_join_absolute_paths;
pub mod rs102_let_underscore_future;
pub mod rs105_manual_unwrap_or_default;
pub mod rs107_misrefactored_assign_op;
pub mod rs109_multi_assignments;
pub mod rs111_mut_range_bound;
pub mod rs113_needless_character_iteration;
pub mod rs115_no_effect_replace;
pub mod rs118_octal_escapes;
pub mod rs119_path_ends_with_ext;
pub mod rs120_permissions_set_readonly_false;
pub mod rs121_pointers_in_nomem_asm_block;
pub mod rs123_rc_clone_in_vec_init;
pub mod rs125_repeat_vec_with_capacity;
pub mod rs126_repr_packed_without_abi;
pub mod rs127_single_range_in_vec_init;
pub mod rs128_size_of_ref;
pub mod rs130_suspicious_assignment_formatting;
pub mod rs131_suspicious_command_arg_space;
pub mod rs132_suspicious_doc_comments;
pub mod rs133_suspicious_else_formatting;
pub mod rs138_suspicious_unary_op_formatting;
pub mod rs139_swap_ptr_to_ref;

use lintbook_core::*;

pub fn lints() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(rs001_absurd_extreme_comparisons::AbsurdExtremeComparisons),
        Box::new(rs002_almost_swapped::AlmostSwapped),
        Box::new(rs003_approx_constant::ApproxConstant),
        Box::new(rs004_async_yields_async::AsyncYieldsAsync),
        Box::new(rs013_eq_op::EqOp),
        Box::new(rs014_erasing_op::ErasingOp),
        Box::new(rs016_ifs_same_cond::IfsSameCond),
        Box::new(rs022_inline_fn_without_body::InlineFnWithoutBody),
        Box::new(rs025_invisible_characters::InvisibleCharacters),
        Box::new(rs026_iter_next_loop::IterNextLoop),
        Box::new(rs027_iter_skip_zero::IterSkipZero),
        Box::new(rs028_iterator_step_by_zero::IteratorStepByZero),
        Box::new(rs029_let_underscore_lock::LetUnderscoreLock),
        Box::new(rs032_mem_replace_with_uninit::MemReplaceWithUninit),
        Box::new(rs034_mistyped_literal_suffixes::MistypedLiteralSuffixes),
        Box::new(rs035_modulo_one::ModuloOne),
        Box::new(rs038_non_octal_unix_permissions::NonOctalUnixPermissions),
        Box::new(rs041_option_env_unwrap::OptionEnvUnwrap),
        Box::new(rs046_possible_missing_comma::PossibleMissingComma),
        Box::new(rs050_reversed_empty_ranges::ReversedEmptyRanges),
        Box::new(rs051_self_assignment::SelfAssignment),
        Box::new(rs054_suspicious_splitn::SuspiciousSplitN),
        Box::new(rs059_unit_cmp::UnitCmp),
        Box::new(rs055_transmute_null_to_fn::TransmuteNullToFn),
        Box::new(rs056_transmuting_null::TransmutingNull),
        Box::new(rs057_uninit_assumed_init::UninitAssumedInit),
        Box::new(rs058_uninit_vec::UninitVec),
        Box::new(rs060_unit_hash::UnitHash),
        Box::new(rs063_unused_io_amount::UnusedIoAmount),
        Box::new(rs064_useless_attribute::UselessAttribute),
        Box::new(rs065_vec_resize_to_zero::VecResizeToZero),
        Box::new(rs075_cast_abs_to_unsigned::CastAbsToUnsigned),
        Box::new(rs079_cast_slice_from_raw_parts::CastSliceFromRawParts),
        Box::new(rs081_const_is_empty::ConstIsEmpty),
        Box::new(rs082_crate_in_macro_def::CrateInMacroDef),
        Box::new(rs083_deprecated_clippy_cfg_attr::DeprecatedClippyCfgAttr),
        Box::new(rs087_duplicate_mod::DuplicateMod),
        Box::new(rs088_duplicated_attributes::DuplicatedAttributes),
        Box::new(rs089_empty_docs::EmptyDocs),
        Box::new(rs090_empty_line_after_doc_comments::EmptyLineAfterDocComments),
        Box::new(rs091_empty_line_after_outer_attr::EmptyLineAfterOuterAttr),
        Box::new(rs096_from_raw_with_void_ptr::FromRawWithVoidPtr),
        Box::new(rs101_join_absolute_paths::JoinAbsolutePaths),
        Box::new(rs102_let_underscore_future::LetUnderscoreFuture),
        Box::new(rs105_manual_unwrap_or_default::ManualUnwrapOrDefault),
        Box::new(rs107_misrefactored_assign_op::MisrefactoredAssignOp),
        Box::new(rs109_multi_assignments::MultiAssignments),
        Box::new(rs111_mut_range_bound::MutRangeBound),
        Box::new(rs113_needless_character_iteration::NeedlessCharacterIteration),
        Box::new(rs115_no_effect_replace::NoEffectReplace),
        Box::new(rs118_octal_escapes::OctalEscapes),
        Box::new(rs119_path_ends_with_ext::PathEndsWithExt),
        Box::new(rs120_permissions_set_readonly_false::PermissionsSetReadonlyFalse),
        Box::new(rs121_pointers_in_nomem_asm_block::PointersInNomemAsmBlock),
        Box::new(rs123_rc_clone_in_vec_init::RcCloneInVecInit),
        Box::new(rs125_repeat_vec_with_capacity::RepeatVecWithCapacity),
        Box::new(rs126_repr_packed_without_abi::ReprPackedWithoutAbi),
        Box::new(rs127_single_range_in_vec_init::SingleRangeInVecInit),
        Box::new(rs128_size_of_ref::SizeOfRef),
        Box::new(rs130_suspicious_assignment_formatting::SuspiciousAssignmentFormatting),
        Box::new(rs131_suspicious_command_arg_space::SuspiciousCommandArgSpace),
        Box::new(rs132_suspicious_doc_comments::SuspiciousDocComments),
        Box::new(rs133_suspicious_else_formatting::SuspiciousElseFormatting),
        Box::new(rs138_suspicious_unary_op_formatting::SuspiciousUnaryOpFormatting),
        Box::new(rs139_swap_ptr_to_ref::SwapPtrToRef),
        Box::new(rs092_empty_loop::EmptyLoop),
        Box::new(rs095_four_forward_slashes::FourForwardSlashes),
    ]
}
