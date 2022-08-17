use crate::{impl_query_self_only, prelude::*};

/// A virtual widget use to help write code when you need a widget as a virtual
/// node in `widget!` macro, or hold a place in tree. When it have a child
/// itself will be dropped when build tree, otherwise as a render widget but do
/// nothing.
#[derive(Declare)]
pub struct Void;

impl Render for Void {
  fn perform_layout(&self, _: BoxClamp, ctx: &mut LayoutCtx) -> Size {
    assert_eq!(
      ctx.single_child(),
      None,
      "Void only used to hold a node place."
    );
    Size::zero()
  }

  fn paint(&self, _: &mut PaintingCtx) {}

  fn only_sized_by_parent(&self) -> bool { true }
}

impl crate::prelude::ComposeSingleChild for Void {
  fn compose_single_child(_: StateWidget<Self>, child: Option<Widget>, _: &mut BuildCtx) -> Widget
  where
    Self: Sized,
  {
    if let Some(child) = child {
      child
    } else {
      Void.into_widget()
    }
  }
}

impl Query for Void {
  impl_query_self_only!();
}
