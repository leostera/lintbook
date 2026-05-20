#[derive(Debug)]
#[derive(Clone)]  // This is fine - different attributes
#[allow(dead_code)]
fn normal_function() {}

#[derive(Debug)]
#[derive(Debug)]  // Violation - duplicate derive(Debug)
fn duplicate_derive() {}

#[allow(dead_code)]
#[allow(dead_code)]  // Violation - duplicate allow(dead_code)
fn duplicate_allow() {}

#[cfg(test)]
#[cfg(test)]  // Violation - duplicate cfg(test)
fn duplicate_cfg() {}

#[inline]
#[inline]  // Violation - duplicate inline
fn duplicate_inline() {}

// Struct with duplicated attributes
#[derive(Debug)]
#[derive(Debug)]  // Violation - duplicate derive(Debug)
#[derive(Clone)]  // This is fine - different attribute
struct DuplicatedStruct {
    #[allow(dead_code)]
    #[allow(dead_code)]  // Violation - duplicate on field
    field1: i32,

    #[serde(rename = "field2")]
    field2: String,
}

// Enum with duplicated attributes
#[derive(Debug)]
#[derive(Debug)]  // Violation - duplicate derive(Debug)
enum DuplicatedEnum {
    Variant1,
    Variant2,
}

// Trait with duplicated attributes
#[cfg(feature = "trait")]
#[cfg(feature = "trait")]  // Violation - duplicate cfg
trait DuplicatedTrait {
    fn method(&self);
}

// Implementation with duplicated attributes
#[cfg(test)]
#[cfg(test)]  // Violation - duplicate cfg(test)
impl DuplicatedStruct {
    #[allow(unused)]
    #[allow(unused)]  // Violation - duplicate allow(unused)
    fn method(&self) {}
}

// No violations - all different attributes
#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
#[allow(dead_code)]
#[cfg(test)]
#[inline]
fn many_different_attributes() {}