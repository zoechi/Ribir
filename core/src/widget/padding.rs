use crate::prelude::*;

/// A widget that insets its child by the given padding.
#[stateful]
#[derive(SingleChildWidget, Clone, Declare)]
pub struct Padding {
  #[declare(builtin)]
  pub padding: EdgeInsets,
}

impl RenderWidget for Padding {
  type RO = Self;

  #[inline]
  fn create_render_object(&self) -> Self::RO { self.clone() }

  #[inline]
  fn update_render_object(&self, object: &mut Self::RO, ctx: &mut UpdateCtx) {
    if self.padding != object.padding {
      ctx.mark_needs_layout();
      object.padding = self.padding.clone();
    }
  }
}

impl RenderObject for Padding {
  fn perform_layout(&mut self, clamp: BoxClamp, ctx: &mut RenderCtx) -> Size {
    debug_assert_eq!(ctx.children().count(), 1);

    let thickness = self.padding.thickness();
    let zero = Size::zero();
    let min = (clamp.min - thickness).max(zero);
    let max = (clamp.max - thickness).max(zero);
    // Shrink the clamp of child.
    let child_clamp = BoxClamp { min, max };
    let mut child = ctx.children().next().expect("Margin must have one child");
    let size = child.perform_layout(child_clamp);

    // Expand the size, so the child have padding.
    let size = clamp.clamp(size + thickness);
    child.update_size(size);

    // Update child's children position, let the have a correct position after
    // expanded with padding. padding.
    child.children().for_each(|mut c| {
      let pos = c
        .box_rect()
        .expect("The grandson must performed layout")
        .origin;
      c.update_position(pos + Vector::new(self.padding.left, self.padding.top));
    });
    size
  }

  #[inline]
  fn only_sized_by_parent(&self) -> bool { false }

  #[inline]
  fn paint<'a>(&'a self, _: &mut PaintingContext<'a>) {}
}

impl Padding {
  #[inline]
  pub fn new(padding: EdgeInsets) -> Self { Self { padding } }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn smoke() {
    let widget = declare! {
      Row {
        padding: EdgeInsets::only_left(1.), ..<_>::default(),
        SizedBox { size: Size::new(100., 100.) }
      }
    };

    let mut wnd = window::Window::without_render(widget, Size::new(200., 200.));
    wnd.render_ready();
    let r_tree = wnd.render_tree();
    let padding_widget = r_tree.root().unwrap();

    assert_eq!(
      padding_widget.layout_box_rect(&*r_tree).unwrap(),
      Rect::from_size(Size::new(101., 100.))
    );

    let box_widget = padding_widget.children(&*r_tree).next().unwrap();
    assert_eq!(
      box_widget.layout_box_rect(&*r_tree).unwrap(),
      Rect::from_size(Size::new(101., 100.))
    );

    let child_box = box_widget.children(&*r_tree).next().unwrap();
    assert_eq!(
      child_box.layout_box_rect(&*r_tree).unwrap(),
      Rect::new(Point::new(1., 0.), Size::new(100., 100.))
    );
  }
}
