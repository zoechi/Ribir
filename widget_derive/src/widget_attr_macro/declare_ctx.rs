use crate::{
  error::{DeclareError, DeclareWarning},
  WIDGET_MACRO_NAME,
};

use super::{
  capture_widget, declare_widget::BuiltinFieldWidgets, ribir_suffix_variable, ScopeUsedInfo,
  UsedType, WidgetMacro,
};

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use std::collections::{HashMap, HashSet};
use syn::{
  parse_quote, parse_quote_spanned, spanned::Spanned, visit_mut, visit_mut::VisitMut, Expr, Ident,
  ItemMacro, Member,
};

bitflags::bitflags! {
  pub struct IdType: u16 {
    /// Declared by `id: name`,
    const DECLARE = 0x001;
    /// name provide in `track { ... }`
    const USER_SPECIFY = 0x010;
      /// name pass by outside `widget!` macro.
    const FROM_ANCESTOR = 0x100;
  }
}

#[derive(Default)]
pub struct DeclareCtx {
  /// All name we can use in macro and need to reactive to its change
  pub named_objects: HashMap<Ident, IdType, ahash::RandomState>,
  pub current_used_info: ScopeUsedInfo,
  /// name object has be used.
  used_widgets: HashSet<Ident, ahash::RandomState>,
  analyze_stack: Vec<Vec<LocalVariable>>,
  /// Some builtin widget (like margin, padding) implicit defined by user,
  /// shared the `id` with host widget in user perspective.
  user_perspective_name: HashMap<Ident, Ident>,
}

#[derive(Debug, Clone)]
struct LocalVariable {
  name: Ident,
  alias_of_name: Option<Ident>,
}

impl VisitMut for DeclareCtx {
  fn visit_expr_mut(&mut self, expr: &mut Expr) {
    match expr {
      Expr::Macro(m) => {
        let mac = &m.mac;
        if mac.path.is_ident(WIDGET_MACRO_NAME) {
          *expr = unwrap_expr(self.expand_widget_macro(mac.tokens.clone()));
        } else {
          visit_mut::visit_expr_macro_mut(self, m);
        }
      }
      Expr::Path(p) => {
        visit_mut::visit_expr_path_mut(self, p);
        if let Some(name) = p.path.get_ident() {
          if let Some(name) = self.find_named_widget(name).cloned() {
            self.add_used_widget(name, UsedType::USED)
          }
        }
      }
      Expr::Closure(c) => {
        let mut outside_used = self.current_used_info.take();
        visit_mut::visit_expr_closure_mut(self, c);
        let mut overwrite_inner_used = UsedType::CAPTURE;
        if c.capture.is_some() {
          if let Some(refs) = self.current_used_info.refs_tokens() {
            let body = &c.body;
            c.body = parse_quote_spanned! { body.span() => { #(#refs)*  #body }};
          }
          if let Some(all) = self.current_used_info.all_widgets() {
            let captures = all.map(capture_widget);
            *expr = parse_quote_spanned! {c.span() => {
              #(#captures)*
              #c
            }}
          }
          overwrite_inner_used = UsedType::MOVE_CAPTURE;
        }

        self.current_used_info.iter_mut().for_each(|(_, info)| {
          info.used_type = overwrite_inner_used;
        });

        outside_used.merge(&self.current_used_info);
        self.current_used_info = outside_used;
      }
      _ => {
        visit_mut::visit_expr_mut(self, expr);
      }
    }
  }

  fn visit_stmt_mut(&mut self, i: &mut syn::Stmt) {
    if let syn::Stmt::Item(syn::Item::Macro(ItemMacro { ident: None, mac, semi_token, .. })) = i {
      let mut expr_to_stmt = |expr| {
        if let Some(semi) = semi_token.take() {
          syn::Stmt::Semi(expr, semi)
        } else {
          syn::Stmt::Expr(expr)
        }
      };
      if mac.path.is_ident(WIDGET_MACRO_NAME) {
        let res = self.expand_widget_macro(mac.tokens.clone());
        *i = expr_to_stmt(unwrap_expr(res));
        return;
      }
    }
    visit_mut::visit_stmt_mut(self, i);
  }

  fn visit_expr_field_mut(&mut self, f_expr: &mut syn::ExprField) {
    if let Some(mut name) = self.expr_find_name_widget(&f_expr.base).cloned() {
      if let Member::Named(ref field_name) = f_expr.member {
        if let Some(suffix) = BuiltinFieldWidgets::as_builtin_widget(field_name) {
          if self
            .named_objects
            .get(&name)
            .map_or(false, |t| t.contains(IdType::DECLARE))
          {
            name.set_span(name.span().join(field_name.span()).unwrap());
            let wrap_name = ribir_suffix_variable(&name, &suffix.to_string());
            *f_expr.base = parse_quote! { #wrap_name };
            self.add_used_widget(wrap_name, UsedType::USED);
            return;
          }
        }
      }
    }
    visit_mut::visit_expr_field_mut(self, f_expr);
  }

  fn visit_expr_assign_mut(&mut self, assign: &mut syn::ExprAssign) {
    visit_mut::visit_expr_assign_mut(self, assign);

    let local_alias = self.expr_find_name_widget(&assign.left).and_then(|local| {
      self
        .expr_find_name_widget(&assign.right)
        .map(|named| (local.clone(), named.clone()))
    });
    if let Some((local, named)) = local_alias {
      let local_var = self
        .analyze_stack
        .iter_mut()
        .rev()
        .flat_map(|locals| locals.iter_mut().rev())
        .find(|v| v.name == local);
      if let Some(local_var) = local_var {
        local_var.alias_of_name = Some(named);
      }
    }
  }

  fn visit_block_mut(&mut self, i: &mut syn::Block) {
    let mut ctx = self.stack_push();
    visit_mut::visit_block_mut(&mut *ctx, i);
  }

  fn visit_item_const_mut(&mut self, i: &mut syn::ItemConst) {
    visit_mut::visit_item_const_mut(self, i);

    self.analyze_stack.last_mut().unwrap().push(LocalVariable {
      name: i.ident.clone(),
      alias_of_name: None,
    });
  }

  fn visit_local_mut(&mut self, local: &mut syn::Local) {
    visit_mut::visit_local_mut(self, local);

    if let Some((_, init)) = &local.init {
      let right_name = self.expr_find_name_widget(init).cloned();
      let var_name = self.analyze_stack.last_mut().unwrap().last_mut();
      // var_name maybe none if
      // `let _ = xxx`
      if let Some(var) = var_name {
        var.alias_of_name = right_name;
      }
    }
  }

  fn visit_expr_block_mut(&mut self, i: &mut syn::ExprBlock) {
    let mut ctx = self.stack_push();
    visit_mut::visit_expr_block_mut(&mut *ctx, i);
  }

  fn visit_expr_for_loop_mut(&mut self, i: &mut syn::ExprForLoop) {
    let mut ctx = self.stack_push();
    visit_mut::visit_expr_for_loop_mut(&mut *ctx, i);
  }

  fn visit_expr_loop_mut(&mut self, i: &mut syn::ExprLoop) {
    let mut ctx = self.stack_push();
    visit_mut::visit_expr_loop_mut(&mut *ctx, i);
  }

  fn visit_expr_if_mut(&mut self, i: &mut syn::ExprIf) {
    let mut ctx = self.stack_push();
    visit_mut::visit_expr_if_mut(&mut *ctx, i);
  }

  fn visit_arm_mut(&mut self, i: &mut syn::Arm) {
    let mut ctx = self.stack_push();
    visit_mut::visit_arm_mut(&mut *ctx, i);
  }

  fn visit_expr_unsafe_mut(&mut self, i: &mut syn::ExprUnsafe) {
    let mut ctx = self.stack_push();
    visit_mut::visit_expr_unsafe_mut(&mut *ctx, i);
  }

  fn visit_expr_while_mut(&mut self, i: &mut syn::ExprWhile) {
    let mut ctx = self.stack_push();
    visit_mut::visit_expr_while_mut(&mut *ctx, i);
  }

  fn visit_pat_ident_mut(&mut self, i: &mut syn::PatIdent) {
    visit_mut::visit_pat_ident_mut(self, i);

    self
      .analyze_stack
      .last_mut()
      .unwrap_or_else(|| {
        panic!(
          "Crash when visit `{}`, stack should not be empty, at {}:{}:{}",
          quote! { #i },
          file!(),
          line!(),
          column!()
        )
      })
      .push(LocalVariable {
        name: i.ident.clone(),
        alias_of_name: None,
      });
  }
}

impl DeclareCtx {
  pub fn id_collect(&mut self, d: &WidgetMacro) -> super::Result<()> {
    d.object_names_iter().try_for_each(|(name, track)| {
      if self.named_objects.contains_key(name) {
        Err(DeclareError::DuplicateID([(*name).clone(), name.clone()]))
      } else {
        self.named_objects.insert(name.clone(), track);
        Ok(())
      }
    })
  }

  pub fn is_used(&self, name: &Ident) -> bool { self.used_widgets.contains(name) }

  pub fn user_perspective_name(&self, name: &Ident) -> Option<&Ident> {
    self.user_perspective_name.get(name)
  }

  pub fn add_user_perspective_pair(&mut self, def_name: Ident, show_name: Ident) {
    self.user_perspective_name.insert(def_name, show_name);
  }

  pub fn take_current_used_info(&mut self) -> ScopeUsedInfo { self.current_used_info.take() }

  pub fn unused_id_warning(&self) -> impl Iterator<Item = DeclareWarning> {
    self
      .named_objects
      .iter()
      .filter(|(id, ty)| {
        !self.used_widgets.contains(id)
          && !ty.contains(IdType::FROM_ANCESTOR)
          && !id.to_string().starts_with('_')
      })
      .map(|(id, _)| DeclareWarning::UnusedName(id))
  }

  pub fn stack_push(&mut self) -> StackGuard { StackGuard::new(self) }

  // return the name of widget that `ident` point to if it's have.
  pub fn find_named_widget<'a>(&'a self, ident: &'a Ident) -> Option<&'a Ident> {
    self
      .analyze_stack
      .iter()
      .rev()
      .flat_map(|local| local.iter().rev())
      .find(|v| &v.name == ident)
      .and_then(|v| v.alias_of_name.as_ref())
      .or_else(|| self.named_objects.contains_key(ident).then(|| ident))
  }

  pub fn expr_find_name_widget<'a>(&'a self, expr: &'a Expr) -> Option<&'a Ident> {
    if let Expr::Path(syn::ExprPath { path, .. }) = expr {
      path
        .get_ident()
        .and_then(|name| self.find_named_widget(name))
    } else {
      None
    }
  }

  pub fn add_used_widget(&mut self, name: Ident, used_type: UsedType) {
    if let Some(u) = self.user_perspective_name.get(&name) {
      self.used_widgets.insert(u.clone());
    }
    self.used_widgets.insert(name.clone());
    self.current_used_info.add_used(name, used_type);
  }

  fn expand_widget_macro(&mut self, tokens: TokenStream) -> syn::Result<Expr> {
    let mut widget_macro: WidgetMacro = syn::parse2(tokens)?;
    let mut new_ctx = DeclareCtx {
      analyze_stack: self.analyze_stack.clone(),
      ..<_>::default()
    };
    // all named objects should as outside define for embed `widget!` macro.
    self.named_objects.iter().for_each(|(name, id_ty)| {
      let ty = *id_ty | IdType::FROM_ANCESTOR;
      new_ctx.named_objects.insert(name.clone(), ty);
    });

    let tokens = widget_macro
      .gen_tokens(&mut new_ctx)
      .unwrap_or_else(|err| err.into_compile_error());

    // inner `widget!` used means need be captured.
    new_ctx.used_widgets.iter().for_each(|name| {
      if self.named_objects.contains_key(name) || self.user_perspective_name.contains_key(name) {
        self.add_used_widget(name.clone(), UsedType::MOVE_CAPTURE)
      }
    });
    let inner_captures = new_ctx
      .used_widgets
      .iter()
      .filter(|w| self.named_objects.contains_key(w) || self.user_perspective_name.contains_key(w))
      .map(capture_widget);
    let tokens = quote_spanned!(tokens.span()=> {#(#inner_captures)* #tokens} );

    syn::parse2(tokens)
  }
}

fn unwrap_expr(res: syn::Result<Expr>) -> Expr {
  match res {
    Ok(expr) => expr,
    Err(e) => {
      let tokens = e.into_compile_error();
      parse_quote!(#tokens)
    }
  }
}

pub struct StackGuard<'a> {
  ctx: &'a mut DeclareCtx,
}

pub struct CaptureScopeGuard<'a> {
  ctx: &'a mut DeclareCtx,
}

impl<'a> StackGuard<'a> {
  pub fn new(ctx: &'a mut DeclareCtx) -> Self {
    ctx.analyze_stack.push(vec![]);
    StackGuard { ctx }
  }
}

impl<'a> Drop for StackGuard<'a> {
  fn drop(&mut self) { self.ctx.analyze_stack.pop(); }
}

impl<'a> std::ops::Deref for StackGuard<'a> {
  type Target = DeclareCtx;

  fn deref(&self) -> &Self::Target { self.ctx }
}

impl<'a> std::ops::DerefMut for StackGuard<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target { self.ctx }
}

impl<'a> std::ops::Deref for CaptureScopeGuard<'a> {
  type Target = DeclareCtx;

  fn deref(&self) -> &Self::Target { self.ctx }
}

impl<'a> std::ops::DerefMut for CaptureScopeGuard<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target { self.ctx }
}
