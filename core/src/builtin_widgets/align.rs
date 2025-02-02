use crate::prelude::*;

/// A enum that describe how widget align to its box.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
  /// The children are aligned to the start edge of the box provided by parent.
  /// The same as [`HAlign::Left`]! if direction is horizontal and
  /// [`VAlign::Top`]! if direction is vertical.
  #[default]
  Start,
  /// The children are aligned to the center of the line of the box provide by
  /// parent. The same as [`HAlign::Center`]! if direction is horizontal and
  /// [`VAlign::Center`]! if direction is vertical.
  Center,
  /// The children are aligned to the start edge of the box provided by parent.
  /// The same as [`HAlign::Right`]! if direction is horizontal and
  /// [`VAlign::Bottom`]! if direction is vertical.
  End,
  /// Require the children to fill the whole box of one axis. This causes the
  /// constraints passed to the children to be tight. The same as
  /// [`HAlign::Stretch`]! if direction is horizontal and [`VAlign::Stretch`]!
  /// if direction is vertical.
  Stretch,
}

/// A enum that describe how widget align to its box in x-axis.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HAlign {
  /// The children are aligned to the left edge of the box provided by parent.
  #[default]
  Left,
  /// The children are aligned to the x-center of the box provide by parent.
  Center,
  /// The children are aligned to the right edge of the box provided by parent.
  Right,
  /// Require the children to fill the whole box in x-axis. This causes the
  /// constraints passed to the children to be tight.
  Stretch,
}

/// A enum that describe how widget align to its box in y-axis.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VAlign {
  #[default]
  /// The children are aligned to the top edge of the box provided by parent.
  Top,
  /// The children are aligned to the y-center of the box provide by parent.
  Center,
  /// The children are aligned to the bottom edge of the box provided by parent.
  Bottom,
  /// Require the children to fill the whole box in y-axis. This causes the
  /// constraints passed to the children to be tight.
  Stretch,
}

/// A widget that align its child in x-axis, base on child's width.
#[derive(Declare, SingleChild)]
pub struct HAlignWidget {
  #[declare(default, builtin)]
  pub h_align: HAlign,
}

/// A widget that align its child in y-axis, base on child's height.
#[derive(Declare, SingleChild)]
pub struct VAlignWidget {
  #[declare(default, builtin)]
  pub v_align: VAlign,
}

impl Render for HAlignWidget {
  fn perform_layout(&self, mut clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
    let align: Align = self.h_align.into();
    if align == Align::Stretch {
      clamp.min.width = clamp.max.width;
    } else {
      clamp.min.width = 0.;
    }
    let box_width = clamp.max.width;
    let mut layouter = ctx.assert_single_child_layouter();
    let child_size = layouter.perform_widget_layout(clamp);
    let x = align.align_value(child_size.width, box_width);
    layouter.update_position(Point::new(x, 0.));
    Size::new((child_size.width + x).max(0.), child_size.height)
  }

  fn paint(&self, _: &mut PaintingCtx) {}

  #[inline]
  fn hit_test(&self, _: &HitTestCtx, _: Point) -> HitTest {
    HitTest { hit: false, can_hit_child: true }
  }
}

impl Query for HAlignWidget {
  crate::impl_query_self_only!();
}

impl Render for VAlignWidget {
  fn perform_layout(&self, mut clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
    let mut layouter = ctx.assert_single_child_layouter();
    let align: Align = self.v_align.into();
    if align == Align::Stretch {
      clamp.min.height = clamp.max.height;
    } else {
      clamp.min.height = 0.;
    }
    let box_height = clamp.max.height;
    let child_size = layouter.perform_widget_layout(clamp);
    let y = align.align_value(child_size.height, box_height);
    layouter.update_position(Point::new(0., y));
    Size::new(child_size.width, (child_size.height + y).max(0.))
  }

  fn paint(&self, _: &mut PaintingCtx) {}

  #[inline]
  fn hit_test(&self, _: &HitTestCtx, _: Point) -> HitTest {
    HitTest { hit: false, can_hit_child: true }
  }
}

impl Query for VAlignWidget {
  crate::impl_query_self_only!();
}

impl Align {
  pub fn align_value(self, child_size: f32, box_size: f32) -> f32 {
    match self {
      Align::Center => (box_size - child_size) / 2.,
      Align::End => box_size - child_size,
      _ => 0.,
    }
  }
}

impl From<HAlign> for Align {
  #[inline]
  fn from(h: HAlign) -> Self {
    match h {
      HAlign::Left => Align::Start,
      HAlign::Center => Align::Center,
      HAlign::Right => Align::End,
      HAlign::Stretch => Align::Stretch,
    }
  }
}

impl From<VAlign> for Align {
  #[inline]
  fn from(h: VAlign) -> Self {
    match h {
      VAlign::Top => Align::Start,
      VAlign::Center => Align::Center,
      VAlign::Bottom => Align::End,
      VAlign::Stretch => Align::Stretch,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::test::*;

  use super::*;
  const CHILD_SIZE: Size = Size::new(10., 10.);
  const WND_SIZE: Size = Size::new(100., 100.);

  #[test]
  fn h_align() {
    fn test_case(h_align: HAlign, expect: Rect) {
      let w = widget! {
        HAlignWidget {
          h_align,
          MockBox { size: CHILD_SIZE }
        }
      };

      expect_layout_result(
        w,
        Some(WND_SIZE),
        &[
          LayoutTestItem {
            path: &[0],
            expect: ExpectRect::from_size(Size::new(expect.max_x(), expect.max_y())),
          },
          LayoutTestItem {
            path: &[0, 0],
            expect: ExpectRect::from_rect(expect),
          },
        ],
      );
    }

    test_case(HAlign::Left, Rect::new(Point::zero(), CHILD_SIZE));
    test_case(HAlign::Center, Rect::new(Point::new(45., 0.), CHILD_SIZE));
    test_case(HAlign::Right, Rect::new(Point::new(90., 0.), CHILD_SIZE));
    test_case(
      HAlign::Stretch,
      Rect::new(Point::zero(), Size::new(100., 10.)),
    );
  }

  #[test]
  fn v_align() {
    fn test_case(v_align: VAlign, expect: Rect) {
      let w = widget! {
        VAlignWidget {
          v_align,
          MockBox { size: CHILD_SIZE }
        }
      };

      expect_layout_result(
        w,
        Some(WND_SIZE),
        &[
          LayoutTestItem {
            path: &[0],
            expect: ExpectRect::from_size(Size::new(expect.max_x(), expect.max_y())),
          },
          LayoutTestItem {
            path: &[0, 0],
            expect: ExpectRect::from_rect(expect),
          },
        ],
      );
    }

    test_case(VAlign::Top, Rect::new(Point::zero(), CHILD_SIZE));
    test_case(VAlign::Center, Rect::new(Point::new(0., 45.), CHILD_SIZE));
    test_case(VAlign::Bottom, Rect::new(Point::new(0., 90.), CHILD_SIZE));
    test_case(
      VAlign::Stretch,
      Rect::new(Point::zero(), Size::new(10., 100.)),
    );
  }
}
