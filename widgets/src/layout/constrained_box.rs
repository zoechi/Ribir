use ribir_core::{impl_query_self_only, prelude::*};

/// a widget that imposes additional constraints clamp on its child.
#[derive(SingleChild, Declare, Clone)]
pub struct ConstrainedBox {
  pub clamp: BoxClamp,
}

impl Render for ConstrainedBox {
  fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
    let max = clamp.max.min(self.clamp.max);
    let min = clamp.min.max(self.clamp.min).min(max);
    ctx.assert_perform_single_child_layout(BoxClamp { min, max })
  }

  #[inline]
  fn only_sized_by_parent(&self) -> bool { false }

  #[inline]
  fn paint(&self, _: &mut PaintingCtx) {}
}

impl Query for ConstrainedBox {
  impl_query_self_only!();
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::prelude::*;
  use ribir_core::test::*;

  #[test]
  fn expand_one_axis() {
    let w = widget! {
      Container {
        size: Size::new(256., 50.),
        ConstrainedBox {
          clamp: BoxClamp::EXPAND_X,
          Container {
            size: Size::new(128., 20.),
          }
        }
      }
    };
    expect_layout_result_with_theme(
      w,
      None,
      material::purple::light(),
      &[LayoutTestItem {
        path: &[0, 0],
        expect: ExpectRect {
          x: Some(0.),
          y: Some(0.),
          width: Some(256.),
          height: Some(20.),
        },
      }],
    );
  }

  #[test]
  fn expand_both() {
    let w = widget! {
      Container {
        size: Size::new(256., 50.),
        ConstrainedBox {
          clamp: BoxClamp::EXPAND_BOTH,
          Container {
            size: Size::new(128., 20.),
          }
        }
      }
    };
    expect_layout_result_with_theme(
      w,
      None,
      material::purple::light(),
      &[LayoutTestItem {
        path: &[0, 0],
        expect: ExpectRect {
          x: Some(0.),
          y: Some(0.),
          width: Some(256.),
          height: Some(50.),
        },
      }],
    );
  }
}
