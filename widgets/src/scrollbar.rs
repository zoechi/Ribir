use std::time::Duration;

use crate::layout::{Container, Stack};
use ribir_core::prelude::*;

/// A control widget that enables the user to access horizontal parts child that
/// is larger than the box rect.
#[derive(Declare, Clone)]
pub struct HScrollBar {
  /// Scrolled pixels of child content.
  #[declare(default)]
  pub offset: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollBarTheme {
  /// The min size of the thumb have.
  pub thumb_min_size: f32,
  /// The thickness of scrollbar element.
  pub thickness: f32,
  /// The brush of the scrollbar track.
  pub track_brush: Brush,
}

/// Compose style that use to decoration the thumb of horizontal scrollbar,
/// overwrite it when init theme.
#[derive(Debug, Declare)]
pub struct HScrollBarThumbStyle {
  pub offset: f32,
}

impl ComposeStyle for HScrollBarThumbStyle {
  type Host = Widget;
  #[inline]
  fn compose_style(this: Stateful<Self>, host: Widget) -> Widget {
    widget! {
      states { this }
      DynWidget { left_anchor: this.offset, dyns: host }
    }
  }
}

/// Compose style that use to decoration the thumb of vertical scrollbar,
/// overwrite it when init theme.
#[derive(Debug, Declare)]
pub struct VScrollBarThumbStyle {
  pub offset: f32,
}

impl ComposeStyle for VScrollBarThumbStyle {
  type Host = Widget;
  #[inline]
  fn compose_style(this: Stateful<Self>, host: Widget) -> Widget {
    widget! {
      states { this }
      DynWidget {
        top_anchor: this.offset,
        dyns: host
      }
    }
  }
}

impl ComposeChild for HScrollBar {
  type Child = Widget;
  fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
    widget! {
      states { this: this.into_writable() }
      Stack {
        ScrollableWidget {
          id: scrolling,
          scrollable: Scrollable::X,
          scroll_pos: Point::new(this.offset, 0.),
          v_align: VAlign::Stretch,
          h_align: HAlign::Stretch,
          DynWidget { dyns: child }
        }
        HRawScrollbar {
          scrolling: scrolling.clone_stateful(),
          v_align: VAlign::Bottom,
        }
      }
      finally ctx => {
        let_watch!(scrolling.scroll_pos.x)
          .distinct_until_changed()
          .debounce(Duration::ZERO, ctx.wnd_ctx().frame_scheduler())
          .subscribe(move |v| this.offset = v);
      }
    }
  }
}

/// A control widget that enables the user to access vertical parts child that
/// is larger than the box rect.
#[derive(Declare, Clone)]
pub struct VScrollBar {
  /// Scrolled pixels of child content.
  #[declare(default)]
  pub offset: f32,
}

impl ComposeChild for VScrollBar {
  type Child = Widget;
  fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
    widget! {
      states { this: this.into_writable() }
      Stack {
        ScrollableWidget {
          id: scrolling,
          scrollable: Scrollable::Y,
          scroll_pos: Point::new(0., this.offset),
          v_align: VAlign::Stretch,
          h_align: HAlign::Stretch,
          DynWidget { dyns: child }
        }
        VRawScrollbar {
          scrolling: scrolling.clone_stateful(),
          h_align: HAlign::Right
        }
      }
      finally ctx => {
        let_watch!(scrolling.scroll_pos.y)
          .distinct_until_changed()
          .debounce(Duration::ZERO, ctx.wnd_ctx().frame_scheduler())
          .subscribe(move |v| this.offset = v);
      }
    }
  }
}
/// A control widget that enables the user to access horizontal parts child that
/// is larger than the box rect.
#[derive(Declare, Clone)]
pub struct BothScrollbar {
  /// Scrolled pixels of child content.
  #[declare(default)]
  pub offset: Point,
}

impl ComposeChild for BothScrollbar {
  type Child = Widget;
  fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
    widget! {
      states { this: this.into_writable() }
      Stack {
        ScrollableWidget {
          id: scrolling,
          scrollable: Scrollable::Both,
          scroll_pos: this.offset,
          v_align: VAlign::Stretch,
          h_align: HAlign::Stretch,
          DynWidget { dyns: child }
        }
        HRawScrollbar {
          id: h_bar,
          scrolling: scrolling.clone_stateful(),
          v_align: VAlign::Bottom,
          margin: EdgeInsets::only_right(v_bar.layout_width())
        }
        VRawScrollbar {
          id: v_bar,
          scrolling: scrolling.clone_stateful(),
          h_align: HAlign::Right,
          margin: EdgeInsets::only_bottom(h_bar.layout_height())
        }
      }
      finally ctx => {
        let_watch!(scrolling.scroll_pos)
          .distinct_until_changed()
          .debounce(Duration::ZERO, ctx.wnd_ctx().frame_scheduler())
          .subscribe(move |v| this.offset = v);
      }
    }
  }
}

/// A widget that display the horizontal scrolling information of the
/// `scrolling` widget.
#[derive(Declare)]
pub struct HRawScrollbar {
  scrolling: Stateful<ScrollableWidget>,
}

impl Compose for HRawScrollbar {
  fn compose(this: State<Self>) -> Widget {
    let this = this.into_writable();
    let scrolling = this.state_ref().scrolling.clone();

    widget! {
      states { scrolling, this }
      init ctx => {
        let ScrollBarTheme {
          thickness,
          thumb_min_size,
          ref track_brush,
        } = *ScrollBarTheme::of(ctx);
      }

      Stack {
        visible: scrolling.can_scroll(),
        Container {
          id: track_box,
          size: Size::new(f32::MAX, thumb_outline.layout_height()),
          background: track_brush.clone()
        }
        LayoutBox {
          id: thumb_outline,
          HScrollBarThumbStyle{
            offset: {
              let content_width = scrolling.scroll_content_size().width;
              -scrolling.scroll_pos.x * safe_recip(content_width) * track_box.layout_width()
            },
            Container {
              size: {
                let page_width = scrolling.scroll_view_size().width;
                let content_width = scrolling.scroll_content_size().width;
                let width = page_width / content_width * track_box.layout_width();
                Size::new(width.max(thumb_min_size), thickness)
              },
            }
         }
        }
      }
    }
  }
}

/// A widget that display the vertical scrolling information of the
/// `scrolling` widget.
#[derive(Declare)]
pub struct VRawScrollbar {
  scrolling: Stateful<ScrollableWidget>,
}

impl Compose for VRawScrollbar {
  fn compose(this: State<Self>) -> Widget {
    let this = this.into_writable();
    let scrolling = this.state_ref().scrolling.clone();

    widget! {
      states { scrolling, this }
      init ctx => {
        let ScrollBarTheme {
          thickness,
          thumb_min_size,
          ref track_brush
        } = *ScrollBarTheme::of(ctx);
      }

      Stack {
        visible: scrolling.can_scroll(),
        Container {
          id: track_box,
          size: Size::new(thumb_outline.layout_width() , f32::MAX),
          background: track_brush.clone(),
        }
        LayoutBox {
          id: thumb_outline,
          VScrollBarThumbStyle {
            offset: {
              let content_height = scrolling.scroll_content_size().height;
              -scrolling.scroll_pos.y * safe_recip(content_height) * track_box.layout_height()
            },
            Container {
              size: {
                let page_height = scrolling.scroll_view_size().height;
                let content_height = scrolling.scroll_content_size().height;
                let height = page_height / content_height * track_box.layout_height();
                Size::new(thickness, height.max(thumb_min_size))
              },
            }
          }
        }
      }
    }
  }
}

fn safe_recip(v: f32) -> f32 {
  let v = v.recip();
  if v.is_infinite() || v.is_nan() { 0. } else { v }
}

impl CustomTheme for ScrollBarTheme {}

#[cfg(test)]
mod test {
  use crate::{layout::Column, prelude::material};

  use super::*;
  use ribir_core::test::*;

  #[test]
  fn content_expand_so_all_view_can_scroll() {
    let w = widget! {
      Stack {
        HScrollBar {
          Container { size: Size::new(100., 100.) }
        }
        VScrollBar {
          Container { size: Size::new(100., 100.) }
        }
        BothScrollbar {
          Container { size: Size::new(100., 100.) }
        }
      }
    };

    expect_layout_result_with_theme(
      w,
      Some(Size::new(200., 200.)),
      material::purple::light(),
      &[
        LayoutTestItem {
          path: &[0, 0],
          expect: ExpectRect {
            width: Some(200.),
            height: Some(200.),
            ..<_>::default()
          },
        },
        LayoutTestItem {
          path: &[0, 1],
          expect: ExpectRect {
            width: Some(200.),
            height: Some(200.),
            ..<_>::default()
          },
        },
        LayoutTestItem {
          path: &[0, 2],
          expect: ExpectRect {
            width: Some(200.),
            height: Some(200.),
            ..<_>::default()
          },
        },
      ],
    );
  }

  #[test]
  fn scrollable() {
    let offset = Stateful::new(Point::zero());
    let v_offset = Stateful::new(0.);
    let h_offset = Stateful::new(0.);
    let w = widget! {
      states { offset: offset.clone(), v_offset: v_offset.clone(), h_offset: h_offset.clone() }
      Column {
        Container {
          size: Size::new(30., 30.),
          BothScrollbar {
            id: both_bar,
            offset: *offset,
            Container { size: Size::new(100., 100.) }
          }
        }
        Container {
          size: Size::new(30., 30.),
          HScrollBar {
            id: h_bar,
            offset: both_bar.offset.x,
            Container { size: Size::new(100., 100.) }
          }
        }
        Container {
          size: Size::new(30., 30.),
          VScrollBar {
            id: v_bar,
            offset: both_bar.offset.y,
            Container { size: Size::new(100., 100.) }
          }
        }
      }

      finally {
        let_watch!(v_bar.offset)
          .subscribe(move|v| *v_offset = v);
        let_watch!(h_bar.offset)
          .subscribe(move|v| *h_offset = v);
      }
    };

    let ctx = AppContext {
      app_theme: std::rc::Rc::new(material::purple::light()),
      ..<_>::default()
    };
    let mut wnd = Window::mock_window(w, Size::new(1024., 1024.), ctx);
    {
      *offset.state_ref() = Point::new(10., 10.);
    }
    {
      *offset.state_ref() = Point::new(20., 20.);
    }
    wnd.draw_frame();
    assert!(*v_offset.state_ref() == offset.state_ref().y);
    assert!(*h_offset.state_ref() == offset.state_ref().x);
  }
}
