pub(crate) use crate::widget_tree::*;
use crate::{
  context::*,
  prelude::ComposeChild,
  state::{State, Stateful},
};
use ribir_algo::ShareResource;
use ribir_painter::*;
use rxrust::subscription::{BoxSubscription, SubscriptionGuard};

#[doc(hidden)]
pub use std::{
  any::{Any, TypeId},
  marker::PhantomData,
  ops::Deref,
};
use std::{cell::RefCell, rc::Rc};
pub trait Compose: Sized {
  /// Describes the part of the user interface represented by this widget.
  /// Called by framework, should never directly call it.
  fn compose(this: State<Self>) -> Widget;
}

pub struct HitTest {
  pub hit: bool,
  pub can_hit_child: bool,
}

/// RenderWidget is a widget which want to paint something or do a layout to
/// calc itself size and update children positions.
///
/// Render Widget should at least implement one of `Layout` or `Paint`, if all
/// of `as_layout` and `as_paint` return None, the widget will not display.
///
/// If `as_layout` return none, widget size will detected by its single child if
/// it has or as large as possible.
pub trait Render: Query {
  /// Do the work of computing the layout for this widget, and return the
  /// size it need.
  ///
  /// In implementing this function, You are responsible for calling every
  /// children's perform_layout across the `LayoutCtx`
  fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size;

  /// `paint` is a low level trait to help you draw your widget to paint device
  /// across `PaintingCtx::painter` by itself coordinate system. Not care
  /// about children's paint in this method, framework will call children's
  /// paint individual. And framework guarantee always paint parent before
  /// children.
  fn paint(&self, ctx: &mut PaintingCtx);

  /// Whether the constraints from parent are the only input to detect the
  /// widget size, and child nodes' size not affect its size.
  fn only_sized_by_parent(&self) -> bool { false }

  /// Hint if a render maybe paint over its layout boundary.
  fn can_overflow(&self) -> bool { false }

  /// Determines the set of render widgets located at the given position.
  fn hit_test(&self, ctx: &HitTestCtx, pos: Point) -> HitTest {
    let is_hit = hit_test_impl(ctx, pos);
    HitTest { hit: is_hit, can_hit_child: is_hit }
  }

  fn get_transform(&self) -> Option<Transform> { None }
}

pub(crate) fn hit_test_impl(ctx: &HitTestCtx, pos: Point) -> bool {
  ctx.box_rect().map_or(false, |rect| rect.contains(pos))
}

pub enum Widget {
  Compose(Box<dyn for<'r> FnOnce(&'r BuildCtx) -> Widget>),
  Render {
    render: Box<dyn Render>,
    children: Option<Vec<Widget>>,
  },
}

/// A trait to query dynamic type and its inner type on runtime, use this trait
/// to provide type information you want framework know.
pub trait Query {
  /// A type can composed by others, this method query all type(include self)
  /// match the type id, and call the callback one by one. The callback accept
  /// an `& dyn Any` of the target type, and return if it want to continue.
  fn query_all(
    &self,
    type_id: TypeId,
    callback: &mut dyn FnMut(&dyn Any) -> bool,
    order: QueryOrder,
  );
}

#[derive(Clone, Copy)]
pub enum QueryOrder {
  InnerFirst,
  OutsideFirst,
}

/// Trait to detect if a type is match the `type_id`.
pub trait QueryFiler {
  /// query self type by type id, and return a reference of `Any` trait to cast
  /// to target type if type match.
  fn query_filter(&self, type_id: TypeId) -> Option<&dyn Any>;
  /// query self type by type id, and return a mut reference of `Any` trait to
  /// cast to target type if type match.
  fn query_filter_mut(&mut self, type_id: TypeId) -> Option<&mut dyn Any>;
}

/// Convert a widget to `Widget`
pub trait IntoWidget<M: ImplMarker> {
  fn into_widget(self) -> Widget;
}

impl<W: 'static> QueryFiler for W {
  #[inline]
  fn query_filter(&self, type_id: TypeId) -> Option<&dyn Any> {
    (self.type_id() == type_id).then_some(self as &dyn Any)
  }

  #[inline]
  fn query_filter_mut(&mut self, type_id: TypeId) -> Option<&mut dyn Any> {
    ((*self).type_id() == type_id).then_some(self as &mut dyn Any)
  }
}

impl<'a> dyn Render + 'a {
  #[inline]
  pub fn query_all_type<T: Any>(&self, mut callback: impl FnMut(&T) -> bool, order: QueryOrder) {
    self.query_all(
      TypeId::of::<T>(),
      &mut |a: &dyn Any| a.downcast_ref().map_or(true, &mut callback),
      order,
    )
  }

  /// Query the first match type in all type by special order, and call
  /// `callback`
  pub fn query_on_first_type<T: Any>(&self, order: QueryOrder, callback: impl FnOnce(&T)) {
    let mut callback = Some(callback);
    self.query_all_type(
      move |a| {
        let cb = callback.take().expect("should only call once");
        cb(a);
        false
      },
      order,
    );
  }

  pub fn contain_type<T: Any>(&self) -> bool {
    let mut hit = false;
    self.query_all_type(
      |_: &T| {
        hit = true;
        false
      },
      QueryOrder::OutsideFirst,
    );
    hit
  }
}

pub trait ImplMarker {}
/// implement marker means this converter not hope to convert continue.
pub struct SelfImpl;
/// implement marker means this converter can use as a generic bounds to convert
/// continue.
pub struct NotSelf<M>(PhantomData<fn(M)>);

impl ImplMarker for SelfImpl {}
impl<M> ImplMarker for NotSelf<M> {}

impl IntoWidget<SelfImpl> for Widget {
  #[inline]
  fn into_widget(self) -> Widget { self }
}

macro_rules! impl_compose_into_widget {
  ($ty: ty) => {
    impl<C: Compose> IntoWidget<NotSelf<[(); 0]>> for $ty {
      #[inline]
      fn into_widget(self) -> Widget { Compose::compose(State::<C>::from(self)) }
    }
  };
}

impl_compose_into_widget!(State<C>);
impl_compose_into_widget!(C);
impl_compose_into_widget!(Stateful<C>);
// `Stateful<DynWidget<C>>` has its own implementation.
// impl_compose_into_widget!(Stateful<DynWidget<C>>);

impl<R: Render + 'static> IntoWidget<NotSelf<[(); 1]>> for R {
  #[inline]
  fn into_widget(self) -> Widget {
    Widget::Render {
      render: Box::new(self),
      children: None,
    }
  }
}

impl<M1, M2, W> IntoWidget<NotSelf<(M1, M2)>> for State<W>
where
  W: IntoWidget<M1>,
  Stateful<W>: IntoWidget<M2>,
  M1: ImplMarker,
  M2: ImplMarker,
{
  fn into_widget(self) -> crate::widget::Widget {
    match self {
      State::Stateless(w) => w.into_widget(),
      State::Stateful(s) => s.into_widget(),
    }
  }
}

macro_rules! impl_compose_option_child_into_widget {
  ($ty: ty) => {
    impl<T, C> IntoWidget<NotSelf<[(); 2]>> for $ty
    where
      T: ComposeChild<Child = Option<C>> + 'static,
    {
      #[inline]
      fn into_widget(self) -> Widget { ComposeChild::compose_child(State::<T>::from(self), None) }
    }
  };
}

impl_compose_option_child_into_widget!(Stateful<T>);
impl_compose_option_child_into_widget!(T);
// `Stateful<DynWidget<T>>` has its own implementation.
// impl_compose_option_child_into_widget!(Stateful<DynWidget<T>>);

impl<F, R, M> IntoWidget<NotSelf<[M; 3]>> for F
where
  F: FnOnce(&BuildCtx) -> R + 'static,
  R: IntoWidget<M>,
  M: ImplMarker,
{
  #[inline]
  fn into_widget(self) -> Widget { Widget::Compose(Box::new(move |ctx| self(ctx).into_widget())) }
}

#[macro_export]
macro_rules! impl_proxy_query {
  (reverse [$first: expr $(, $rest: expr)*] $($reversed: expr)*) => {
    impl_proxy_query!(reverse [$($rest),*] $first $($reversed)*);
  };
  (reverse [] $($reversed: expr)*) => { $($reversed)* };
  (
    $($self: ident .$name: ident $(($($args: ident),*))?),+
  ) => {
    #[inline]
    fn query_all(
      &self,
      type_id: TypeId,
      callback: &mut dyn FnMut(&dyn Any) -> bool,
      order: QueryOrder,
    ) {
      let mut query_more = true;
      match order {
        QueryOrder::InnerFirst => {
          impl_proxy_query!(reverse
            [$(
              if query_more {
                self.$name $(($($args),*))?
                  .query_all(
                    type_id,
                    &mut |any| {
                      query_more = callback(any);
                      query_more
                    },
                    order,
                  );
              }
            ),+]
          );
          if let Some(a) = self.query_filter(type_id) {
            callback(a);
          }
        }
        QueryOrder::OutsideFirst => {
          if let Some(a) = self.query_filter(type_id) {
            query_more = callback(a);
          }
          if query_more {
            $(
              if query_more {
                self.$name $(($($args),*))?
                  .query_all(
                    type_id,
                    &mut |any| {
                      query_more = callback(any);
                      query_more
                    },
                    order,
                  );
              }
            )+
          }
        }
      }
    }
  };
}

#[macro_export]
macro_rules! impl_query_self_only {
  () => {
    #[inline]
    fn query_all(
      &self,
      type_id: TypeId,
      callback: &mut dyn FnMut(&dyn Any) -> bool,
      _: QueryOrder,
    ) {
      if let Some(a) = self.query_filter(type_id) {
        callback(a);
      }
    }
  };
}

impl<T: Render> Render for ribir_algo::ShareResource<T> {
  #[inline]
  fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
    T::perform_layout(self, clamp, ctx)
  }

  #[inline]
  fn paint(&self, ctx: &mut PaintingCtx) { T::paint(self, ctx) }

  #[inline]
  fn only_sized_by_parent(&self) -> bool { T::only_sized_by_parent(self) }

  #[inline]
  fn can_overflow(&self) -> bool { T::can_overflow(self) }

  #[inline]
  fn hit_test(&self, ctx: &HitTestCtx, pos: Point) -> HitTest { T::hit_test(self, ctx, pos) }

  #[inline]
  fn get_transform(&self) -> Option<Transform> { T::get_transform(self) }
}

impl<T: Query> Query for ShareResource<T> {
  fn query_all(
    &self,
    type_id: TypeId,
    callback: &mut dyn FnMut(&dyn Any) -> bool,
    order: QueryOrder,
  ) {
    (**self).query_all(type_id, callback, order)
  }
}

#[macro_export]
macro_rules! impl_proxy_render {
  ($($proxy: tt)*) => {
    #[inline]
    fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
      self.$($proxy)*.perform_layout(clamp, ctx)
    }

    #[inline]
    fn paint(&self, ctx: &mut PaintingCtx) { self.$($proxy)*.paint(ctx) }

    #[inline]
    fn only_sized_by_parent(&self) -> bool {
      self.$($proxy)*.only_sized_by_parent()
    }

    #[inline]
    fn can_overflow(&self) -> bool { self.$($proxy)*.can_overflow() }

    #[inline]
    fn hit_test(&self, ctx: &HitTestCtx, pos: Point) -> HitTest {
      self.$($proxy)*.hit_test(ctx, pos)
    }

    #[inline]
    fn get_transform(&self) -> Option<Transform> {
      self.$($proxy)*.get_transform()
    }
  };
}

impl<W: Render + 'static> Render for RefCell<W> {
  impl_proxy_render!(borrow());
}

impl<W: Query + 'static> Query for RefCell<W> {
  impl_proxy_query!(self.borrow());
}

impl<W: Render + 'static> Render for Rc<W> {
  impl_proxy_render!(deref());
}

impl<W: Query + 'static> Query for Rc<W> {
  impl_proxy_query!(self.deref());
}

impl Render for Box<dyn Render> {
  impl_proxy_render!(deref());
}

impl Query for Box<dyn Render> {
  impl_proxy_query!(self.deref());
}

impl Query for Vec<SubscriptionGuard<BoxSubscription<'static>>> {
  impl_query_self_only!();
}
