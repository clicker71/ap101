//! COMPILE-TIME ZERO-ALLOCATION BACKSTOP (SYNTACTIC, NOT SOUND).
//!
//! `zero_alloc! { ... }` expands to its inner block and REJECTS AT COMPILE
//! TIME a SMALL, DOCUMENTED DENY-LIST of allocation-introducing expressions:
//!
//! - `String::new` / `String::from`
//! - `Vec::new` / `Vec::with_capacity`
//! - `Box::new`
//! - `vec!`
//! - `format!`
//! - `.to_string()`
//! - `.to_owned()`
//! - `.collect::<Vec<_>>()`
//!
//! # HONEST LIMITATION (READ THIS)
//!
//! THIS IS A SYNTACTIC HEURISTIC, NOT A SOUND PROOF. It catches the COMMON
//! inline `vec!`/`format!` regression a reviewer would miss; it CANNOT see an
//! allocation performed behind a helper call, through a type alias, through a
//! trait object, or via `Box::leak`. A `Vec<T>` appearing in a TYPE ANNOTATION
//! is not itself an allocation and is deliberately NOT rejected (only a
//! `Vec::new`/`Vec::with_capacity` CALL is). The authoritative 0-alloc gate
//! remains the runtime `TestAllocator` window in ferrite-testkit; this macro
//! is the cheap, coverage-independent backstop that runs on EVERY build.
//!
//! The proc-macro is a BUILD-TIME dependency only: it never appears in a
//! release dependency graph.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    spanned::Spanned, visit::Visit, Expr, ExprCall, ExprMacro, ExprMethodCall, GenericArgument,
    Macro, Path, Type,
};

/// An allocation site found in the wrapped block.
struct AllocUse {
    span: Span,
    label: String,
}

/// Rejects the documented deny-list inside the wrapped block.
///
/// See the module docs for the exact list and the honest limitation.
#[proc_macro]
pub fn zero_alloc(input: TokenStream) -> TokenStream {
    let block: syn::Block = match syn::parse::<syn::Block>(input) {
        Ok(block) => block,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut visitor = AllocVisitor { found: Vec::new() };
    visitor.visit_block(&block);

    if let Some(first) = visitor.found.first() {
        let label = &first.label;
        let message = format!(
            "zero_alloc! rejected a heap allocation: `{label}`. This section is \
             declared 0-alloc; the list is documented in ferrite-macros."
        );
        return syn::Error::new(first.span, message)
            .to_compile_error()
            .into();
    }

    let mut output = proc_macro2::TokenStream::new();
    block.to_tokens(&mut output);
    output.into()
}

struct AllocVisitor {
    found: Vec<AllocUse>,
}

impl AllocVisitor {
    /// Records a deny-list hit at the offending token span.
    fn record(&mut self, span: Span, label: &str) {
        self.found.push(AllocUse {
            span,
            label: label.to_string(),
        });
    }
}

impl<'ast> Visit<'ast> for AllocVisitor {
    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            let rendered = render_path(&path.path);
            if let Some(label) = deny_list_call(&rendered) {
                self.record(last_ident_span(&path.path), label);
            }
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_macro(&mut self, mac: &'ast ExprMacro) {
        if let Some(label) = deny_list_macro(&mac.mac) {
            self.record(last_ident_span(&mac.mac.path), label);
        }
        syn::visit::visit_expr_macro(self, mac);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if let Some(label) = deny_list_method(&method, &call.turbofish) {
            self.record(call.method.span(), label);
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

/// Span of the final path segment (the ident the reader sees).
fn last_ident_span(path: &Path) -> Span {
    path.segments
        .last()
        .map_or_else(|| path.span(), |segment| segment.ident.span())
}

/// Canonical `A::B` rendering of a path, ignoring generic arguments.
fn render_path(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Maps a call path to a deny-list label.
fn deny_list_call(rendered: &str) -> Option<&'static str> {
    match rendered {
        "String::new" => Some("String::new"),
        "String::from" => Some("String::from"),
        "Vec::new" => Some("Vec::new"),
        "Vec::with_capacity" => Some("Vec::with_capacity"),
        "Box::new" => Some("Box::new"),
        _ => None,
    }
}

/// Maps a macro invocation to a deny-list label.
fn deny_list_macro(mac: &Macro) -> Option<&'static str> {
    match render_path(&mac.path).as_str() {
        "vec" => Some("vec!"),
        "format" => Some("format!"),
        _ => None,
    }
}

/// Maps `.to_string()` / `.to_owned()` unconditionally, and `.collect()` ONLY
/// when it collects into a `Vec` (`.collect::<Vec<_>>()`), matching the
/// documented list exactly.
fn deny_list_method(
    method: &str,
    turbofish: &Option<syn::AngleBracketedGenericArguments>,
) -> Option<&'static str> {
    match method {
        "to_string" => Some(".to_string()"),
        "to_owned" => Some(".to_owned()"),
        "collect" if collects_into_vec(turbofish) => Some(".collect::<Vec<_>>()"),
        _ => None,
    }
}

/// True when a `.collect::<...>()` turbofish names `Vec` as an argument type.
fn collects_into_vec(turbofish: &Option<syn::AngleBracketedGenericArguments>) -> bool {
    let Some(args) = turbofish else {
        return false;
    };
    args.args.iter().any(|arg| match arg {
        GenericArgument::Type(Type::Path(type_path)) => path_is_named_vec(&type_path.path),
        _ => false,
    })
}

/// True when the final segment of a path is the ident `Vec`.
fn path_is_named_vec(path: &Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "Vec")
}
