//! Implements a dozen of enums to store different widget, and implement the
//! common trait if all enum variable implement it.

use crate::prelude::*;

macro_rules! impl_enum_widget {
  ($name: ident, $($var_ty: ident) ,+ ) => {
    pub enum $name<$($var_ty),+> {
      $($var_ty($var_ty)),+
    }

    impl< $($var_ty: Query),+> Query for $name <$($var_ty),+> {
      fn query_all(
        &self,
        type_id: TypeId,
        callback: &mut dyn FnMut(&dyn Any) -> bool,
        order: QueryOrder,
      ) {
        match self {
          $($name::$var_ty(w) => w.query_all(type_id, callback, order)),+
        }
      }
    }

    impl< $($var_ty: Render),+> Render for $name <$($var_ty),+> {
      fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
        match self {
          $($name::$var_ty(w) => w.perform_layout(clamp, ctx)),+
        }
      }

      fn paint(&self, ctx: &mut PaintingCtx) {
        match self {
          $($name::$var_ty(w) => w.paint(ctx)),+
        }
      }

      fn only_sized_by_parent(&self) -> bool {
        match self {
          $($name::$var_ty(w) => w.only_sized_by_parent()),+
        }
      }

      fn can_overflow(&self) -> bool {
        match self {
          $($name::$var_ty(w) => w.can_overflow()),+
        }
      }

      fn hit_test(&self, ctx: &HitTestCtx, pos: Point) -> HitTest {
        match self {
          $($name::$var_ty(w) => w.hit_test(ctx, pos)),+
        }
      }

      fn get_transform(&self) -> Option<Transform> {
        match self {
          $($name::$var_ty(w) => w.get_transform()),+
        }
      }
    }

    impl<$($var_ty: SingleChild),+> SingleChild for $name <$($var_ty),+> {

    }
    impl<$($var_ty: MultiChild),+> MultiChild for $name <$($var_ty),+> {}
    impl<Child, $($var_ty: ComposeChild<Child=Child>),+> ComposeChild
      for $name <$($var_ty),+> {
      type Child = Child;
      fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
        let w = match this {
          State::Stateless(w) => w,
          State::Stateful(_) =>  {
            unreachable!("Enum widgets only use to store widget, should never convert to stateful.");
          }
        };
        match w {
          $($name::$var_ty(w) => $var_ty::compose_child(w.into(), child)),+
        }
      }
    }
  };
}

impl_enum_widget!(WidgetE2, A, B);
impl_enum_widget!(WidgetE3, A, B, C);
impl_enum_widget!(WidgetE4, A, B, C, D);
impl_enum_widget!(WidgetE5, A, B, C, D, E);
impl_enum_widget!(WidgetE6, A, B, C, D, E, F);
