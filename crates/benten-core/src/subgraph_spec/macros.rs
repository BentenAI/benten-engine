//! Authoring-ergonomics macros for [`super::Spec`].
//!
//! Per Spike F's raw-struct-literal-is-painful finding: the
//! `query!` macro is sugar that produces the **byte-equal** Spec as
//! the raw `Spec::builder()` chain. The macro is a pure syntactic
//! affordance; the semantics are exactly the builder chain.
//!
//! The test pin (`tf3w_combinators_intersect_union_filter_proptest.rs`
//! arm P-4) asserts byte-equality between the macro form and the raw
//! builder.

/// Construct a [`super::Spec`] via a name-value macro form.
///
/// # Syntax
///
/// ```ignore
/// query!(roots: [r1, r2, ...], max_depth: N, allow_labels: [L1, L2, ...])
/// ```
///
/// All fields are optional except `roots` (the Spec is meaningless
/// without an entry CID). The macro returns
/// `Result<Spec, SubgraphSpecError>` — same as
/// [`super::SpecBuilder::build`].
///
/// # Examples
/// ```ignore
/// use benten_core::subgraph_spec::query;
/// let s = query!(roots: [my_cid], max_depth: 3, allow_labels: ["Recipe"])?;
/// ```
#[macro_export]
macro_rules! query {
    (roots: [$($root:expr),* $(,)?], max_depth: $depth:expr, allow_labels: [$($label:expr),* $(,)?] $(,)?) => {{
        let mut __b = $crate::subgraph_spec::Spec::builder().with_max_depth($depth);
        $(__b = __b.with_root($root);)*
        let __labels: ::alloc::vec::Vec<&'static str> = ::alloc::vec![$($label),*];
        if !__labels.is_empty() {
            __b = __b.with_label_allowlist(__labels);
        }
        __b.build()
    }};
    (roots: [$($root:expr),* $(,)?], max_depth: $depth:expr $(,)?) => {{
        let mut __b = $crate::subgraph_spec::Spec::builder().with_max_depth($depth);
        $(__b = __b.with_root($root);)*
        __b.build()
    }};
    (roots: [$($root:expr),* $(,)?] $(,)?) => {{
        let mut __b = $crate::subgraph_spec::Spec::builder();
        $(__b = __b.with_root($root);)*
        __b.build()
    }};
}
