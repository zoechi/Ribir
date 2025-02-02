use ahash::HashSet;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, token::Brace, Expr, Ident};

mod code_gen;
mod desugar;
mod parser;
pub use desugar::Desugared;
pub use parser::{MacroSyntax, StateField};
mod visit_mut;
pub use visit_mut::*;
mod name_used_info;
pub use name_used_info::*;
mod variable_names;
pub use variable_names::*;

use self::desugar::{builtin_obj, NamedObj};
use crate::error::DeclareError;

fn capture_widget(widget: &Ident) -> TokenStream {
  quote_spanned!(widget.span() => let #widget = #widget.clone_stateful();)
}

#[derive(Debug, Clone)]
pub struct TrackExpr {
  pub expr: Expr,
  pub used_name_info: ScopeUsedInfo,
}

pub fn gen_widget_macro(
  input: proc_macro::TokenStream,
  outside_ctx: Option<&mut VisitCtx>,
) -> proc_macro::TokenStream {
  let macro_syntax = parse_macro_input! { input as MacroSyntax };
  let mut desugar = macro_syntax.desugar();

  let states: HashSet<_> = desugar
    .states
    .iter()
    .flat_map(|t| t.states.iter().map(|sf| sf.member.clone()))
    .collect();

  let mut ctx = VisitCtx { states, ..<_>::default() };
  let mut ctx = ctx.stack_push();

  use DeclareError::DuplicateID;
  let errors = &mut desugar.errors;
  if let Some(ref outside_ctx) = outside_ctx {
    ctx.declare_objs.extend(outside_ctx.declare_objs.clone());

    if !outside_ctx.states.is_empty() {
      if !ctx.states.is_empty() {
        ctx.states.iter().for_each(|name| {
          if let Some(other) = outside_ctx.states.get(name) {
            errors.push(DuplicateID([name.clone(), other.clone()]))
          }
        })
      }
      ctx.states.extend(outside_ctx.states.clone());
    }

    ctx.analyze_stack = outside_ctx.analyze_stack.clone();
    // maybe some variable has the same name as variables in `states`, we push
    // states in variable stack to avoid this situation.
    let local_vars = ctx
      .states
      .iter()
      .map(|name| LocalVariable::new(name.clone(), name.clone()))
      .collect();
    ctx.analyze_stack.push(local_vars);
  }

  if let Some(init) = desugar.init.as_mut() {
    ctx.visit_init_stmts_mut(init);
    if let Some(ctx_name) = init.ctx_name.clone() {
      if let Some(other) = ctx.states.get(&ctx_name) {
        errors.push(DuplicateID([ctx_name.clone(), other.clone()]))
      }
    }
  }

  if !desugar.named_objs.is_empty() {
    desugar
      .named_objs
      .objs()
      .map(|obj| (obj.name().clone(), obj.ty().clone()))
      .for_each(|(name, path)| {
        if let Some((other, _)) = ctx.declare_objs.get_key_value(&name) {
          errors.push(DuplicateID([name.clone(), other.clone()]))
        }
        if let Some(other) = ctx.states.get(&name) {
          errors.push(DuplicateID([name.clone(), other.clone()]))
        }
        ctx.declare_objs.insert(name, path);
      });
  };

  if !ctx.declare_objs.is_empty() && !ctx.states.is_empty() {
    ctx.states.iter().for_each(|name| {
      if let Some((other, _)) = ctx.declare_objs.get_key_value(name) {
        errors.push(DuplicateID([name.clone(), other.clone()]))
      }
    })
  }

  ctx.visit_desugared_syntax_mut(&mut desugar);

  ctx
    .used_objs
    .iter()
    .for_each(|(name, UsedInfo { builtin, .. })| {
      // add default builtin widget, which used by others but but declared.
      if let Some(builtin) = builtin {
        if !desugar.named_objs.contains(name) && desugar.named_objs.contains(&builtin.src_name) {
          let BuiltinUsed { src_name, builtin_ty } = builtin;
          let obj = builtin_obj(src_name, builtin_ty, <_>::default());
          desugar.add_named_builtin_obj(src_name.clone(), obj);
        }
      }

      if let Some(obj) = desugar.named_objs.get_mut(name) {
        // named obj used by other should force be stateful
        match obj {
          NamedObj::Host(obj) | NamedObj::Builtin { obj, .. } => obj.stateful = true,
        }
      }
    });

  desugar.collect_warnings(&ctx);
  let mut tokens = quote! {};
  desugar.circle_detect();
  Brace::default().surround(&mut tokens, |tokens| {
    Brace::default().surround(tokens, |tokens| {
      if outside_ctx.is_none() {
        quote! {
          #![allow(
            unused_mut,
            clippy::redundant_clone,
            clippy::clone_on_copy,
            clippy::let_and_return
          )]
        }
        .to_tokens(tokens);
      }
      desugar.gen_tokens(tokens, &ctx);
    });
  });

  if let Some(outside_ctx) = outside_ctx {
    outside_ctx.visit_error_occur |= ctx.visit_error_occur || !desugar.errors.is_empty();
    let used_outsides = ctx
      .used_objs
      .iter()
      .filter(|(name, _)| {
        !desugar.named_objs.contains(name)
          && desugar
            .states
            .as_ref()
            .map_or(true, |track| !track.track_names().any(|n| &n == name))
      })
      .collect::<Vec<_>>();
    if !used_outsides.is_empty() {
      let captures = used_outsides.iter().map(|(name, _)| capture_widget(name));
      tokens = quote! {{
        #(#captures)*
        #tokens
      }};
    }
    used_outsides.into_iter().for_each(|(name, used_info)| {
      outside_ctx.add_used_widget(
        name.clone(),
        used_info.builtin.clone(),
        UsedType::SCOPE_CAPTURE,
      )
    });
  }

  tokens.into()
}

impl TrackExpr {
  pub fn new(expr: Expr) -> Self { Self { expr, used_name_info: <_>::default() } }
}

impl ToTokens for TrackExpr {
  fn to_tokens(&self, tokens: &mut TokenStream) { self.expr.to_tokens(tokens) }
}
