use crate::state::Stateful;
use std::convert::Infallible;

use super::Lerp;
use rxrust::{
  observable,
  ops::box_it::{BoxIt, BoxOp},
  prelude::ObservableExt,
};

/// Property is a value with can be accessed and watch its changes.
pub trait Property {
  type Value: Clone;
  fn get(&self) -> Self::Value;
  fn set(&mut self, v: Self::Value);
  fn shallow_set(&mut self, v: Self::Value);
  fn modifies(&self) -> BoxOp<'static, (), Infallible>;
}

pub trait AnimateProperty: Property {
  fn calc_lerp_value(&self, from: &Self::Value, to: &Self::Value, rate: f32) -> Self::Value;
}

pub struct Prop<T, G, S> {
  target: Stateful<T>,
  getter: G,
  setter: S,
}

#[derive(Clone)]
pub struct LerpProp<P, F> {
  prop: P,
  /// function calc the linearly lerp value by rate, three arguments are
  /// `from` `to` and `rate`, specify `lerp_fn` when the animate state not
  /// implement `Lerp` trait or you want to specify a custom lerp function.
  lerp_fn: F,
}

impl<T, G, S, V> Prop<T, G, S>
where
  G: Fn(&T) -> V,
  S: FnMut(&mut T, V),
  V: Clone,
{
  #[inline]
  pub fn new(target: Stateful<T>, getter: G, setter: S) -> Self { Self { target, getter, setter } }
}

impl<P, F> LerpProp<P, F>
where
  P: Property,
  F: Fn(&P::Value, &P::Value, f32) -> P::Value,
{
  #[inline]
  pub fn new(prop: P, lerp_fn: F) -> Self { Self { prop, lerp_fn } }
}

impl<T, G, S, V> Property for Prop<T, G, S>
where
  G: Fn(&T) -> V,
  S: FnMut(&mut T, V),
  V: Clone,
{
  type Value = V;

  #[inline]
  fn get(&self) -> V { (self.getter)(&*self.target.state_ref()) }

  #[inline]
  fn set(&mut self, v: V) { (self.setter)(&mut *self.target.state_ref(), v); }

  #[inline]
  fn shallow_set(&mut self, v: Self::Value) { (self.setter)(&mut *self.target.shallow_ref(), v); }

  #[inline]
  fn modifies(&self) -> BoxOp<'static, (), Infallible> { self.target.modifies().box_it() }
}

impl<T, G, S> AnimateProperty for Prop<T, G, S>
where
  Self: Property,
  Self::Value: Lerp,
{
  #[inline]

  fn calc_lerp_value(&self, from: &Self::Value, to: &Self::Value, rate: f32) -> Self::Value {
    from.lerp(to, rate)
  }
}

impl<T, G, S, V> Prop<T, G, S>
where
  Self: Property<Value = V>,
  G: Fn(&T) -> V + Clone + 'static,
  V: PartialEq + Clone + 'static,
  T: 'static,
{
  pub fn changes(&self) -> BoxOp<'static, V, Infallible> {
    let target = self.target.clone();
    let getter = self.getter.clone();
    self
      .modifies()
      .map(move |_| getter(&*target.state_ref()))
      .distinct_until_changed()
      .box_it()
  }
}

impl<P, F> Property for LerpProp<P, F>
where
  P: Property,
{
  type Value = P::Value;

  #[inline]
  fn get(&self) -> Self::Value { self.prop.get() }

  #[inline]
  fn set(&mut self, v: Self::Value) { self.prop.set(v) }

  #[inline]
  fn shallow_set(&mut self, v: Self::Value) { self.prop.shallow_set(v) }

  #[inline]
  fn modifies(&self) -> BoxOp<'static, (), Infallible> { self.prop.modifies() }
}

impl<P, F> AnimateProperty for LerpProp<P, F>
where
  P: Property,
  F: Fn(&P::Value, &P::Value, f32) -> P::Value + Clone,
{
  #[inline]
  fn calc_lerp_value(&self, from: &Self::Value, to: &Self::Value, rate: f32) -> Self::Value {
    (self.lerp_fn)(from, to, rate)
  }
}

impl<T, G, S, F, V> LerpProp<Prop<T, G, S>, F>
where
  Prop<T, G, S>: Property<Value = V>,
  V: PartialEq + Clone + 'static,
  G: Fn(&T) -> V + Clone + 'static,
  T: 'static,
{
  #[inline]
  pub fn changes(&self) -> BoxOp<'static, V, Infallible> { self.prop.changes() }
}

macro_rules! impl_tuple_property {
  ($ty: ident, $idx: tt $(,$other_ty: ident, $other_idx: tt)*) => {
    impl_tuple_property!({$ty, $idx} $($other_ty, $other_idx),*);
  };
  (
    {$($ty: ident, $idx: tt),+}
    $next_ty: ident, $next_idx: tt
    $(,$other_ty: ident, $other_idx: tt)*
  ) => {
      impl_tuple_property!({$($ty, $idx),+});
      impl_tuple_property!(
        {$($ty, $idx,)+ $next_ty, $next_idx }
        $($other_ty, $other_idx),*
      );
  };
  ({ $($ty: ident, $idx:tt),+})  => {
    impl<$($ty),+> Property for ($($ty,)+)
    where
      $($ty: Property + 'static,)+
      $($ty::Value: 'static),+
    {
      type Value = ($($ty::Value,)+);

      #[inline]
      fn get(&self) -> Self::Value {
        ($(self.$idx.get(),)+)
      }

      #[inline]
      fn set(&mut self, v: Self::Value) {
        $(self.$idx.set(v.$idx);)+
      }

      #[inline]
      fn shallow_set(&mut self, v: Self::Value) {
        $(self.$idx.shallow_set(v.$idx);)+
       }

      #[inline]
      fn modifies(&self) -> BoxOp<'static, (), Infallible> {
        observable::from_iter([$(self.$idx.modifies()),+])
          .merge_all(usize::MAX)
          .box_it()
      }
    }

    impl<$($ty),+> AnimateProperty for ($($ty,)+)
    where
      Self: Property<Value= ($($ty::Value,)+)>,
      $($ty: AnimateProperty),+
    {
      fn calc_lerp_value(&self, from: &Self::Value, to: &Self::Value, rate: f32) -> Self::Value{
        ($(self.$idx.calc_lerp_value(&from.$idx, &to.$idx, rate),)+)
      }
    }
  }
}

impl_tuple_property! {T0, 0, T1, 1, T2, 2, T3, 3, T4, 4, T5, 5, T6, 6, T7, 7,T8, 8, T9, 9,
  T10, 10, T11, 11, T12, 12, T13, 13, T14, 14, T15, 15, T16, 16, T17, 17,T18, 18, T19, 19,
  T20, 20, T21, 21, T22, 22, T23, 23, T24, 24, T25, 25, T26, 26, T27, 27,T28, 28, T29, 29,
  T30, 30, T31, 31
}

impl<T, G: Clone, S: Clone> Clone for Prop<T, G, S> {
  fn clone(&self) -> Self {
    Prop {
      target: self.target.clone(),
      getter: self.getter.clone(),
      setter: self.setter.clone(),
    }
  }
}
