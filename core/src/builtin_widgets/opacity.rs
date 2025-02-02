use crate::impl_query_self_only;
use crate::prelude::*;

#[derive(Declare, Default, Clone, SingleChild)]
pub struct Opacity {
  #[declare(builtin)]
  pub opacity: f32,
}

impl Query for Opacity {
  impl_query_self_only!();
}

impl Render for Opacity {
  #[inline]
  fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
    ctx.assert_perform_single_child_layout(clamp)
  }

  fn paint(&self, ctx: &mut PaintingCtx) { ctx.painter().apply_alpha(self.opacity); }

  fn only_sized_by_parent(&self) -> bool { false }

  fn can_overflow(&self) -> bool { self.opacity > 0. }
}
