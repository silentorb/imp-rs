//! Fixed core subtype relation. Spec: imp-spec/docs/packages/imp-typecheck/type-system.md

/// Returns true when `sub_id` is the same as `super_id` or a declared direct subtype.
pub fn is_subtype_id(sub_id: &str, super_id: &str) -> bool {
    if sub_id == super_id {
        return true;
    }
    matches!(
        (sub_id, super_id),
        ("list", "collection") | ("set", "collection")
    )
}
