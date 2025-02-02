use crate::prelude::{svgs, Icon, Row, Text};
use ribir_core::prelude::*;

/// Represents a control that a user can select and clear.
#[derive(Clone, Declare)]
pub struct Checkbox {
  #[declare(default)]
  pub checked: bool,
  #[declare(default)]
  pub indeterminate: bool,
}

#[derive(Clone)]
pub struct CheckBoxTheme {
  /// The size of the checkbox icon.
  pub size: Size,
  /// The text style of the checkbox label.
  pub label_style: CowArc<TextStyle>,
}

#[derive(Clone, Declare)]
pub struct CheckBoxStyle {
  #[declare(default=Palette::of(ctx).primary())]
  pub color: Color,
}

impl Checkbox {
  pub fn switch_check(&mut self) {
    if self.indeterminate {
      self.indeterminate = false;
      self.checked = false;
    } else {
      self.checked = !self.checked;
    }
  }
}

/// Describe label position before or after purpose widget.`
#[derive(Default)]
pub enum Position {
  Before,
  #[default]
  After,
}

#[derive(Declare)]
pub struct CheckboxLabel {
  pub desc: CowArc<str>,
  pub position: Position,
}

impl ComposeStyle for CheckBoxStyle {
  type Host = Widget;
  #[inline]
  fn compose_style(_: Stateful<Self>, style: Self::Host) -> Widget { style }
}

impl ComposeChild for Checkbox {
  type Child = Option<CheckboxLabel>;

  fn compose_child(this: State<Self>, label: Self::Child) -> Widget {
    let this = this.into_writable();
    let mut checkbox = widget! {
      states { this: this.clone() }
      init ctx => { let theme = CheckBoxTheme::of(ctx); }
      CheckBoxStyle { Icon {
        size: theme.size,
        DynWidget {
          dyns: {
            if this.indeterminate {
              svgs::INDETERMINATE_CHECK_BOX
            } else if this.checked {
              svgs::CHECK_BOX
            } else {
              svgs::CHECK_BOX_OUTLINE_BLANK
            }
          }
        }
      }
    }};

    if let Some(CheckboxLabel { desc, position }) = label {
      let label = widget! {
        init ctx => { let theme = CheckBoxTheme::of(ctx); }
        Text { text: desc, style: theme.label_style.clone() }
      };
      checkbox = match position {
        Position::Before => {
          widget! { Row { DynWidget { dyns: [label, checkbox] } } }
        }
        Position::After => {
          widget! { Row { DynWidget { dyns: [checkbox, label] } } }
        }
      };
    }

    widget! {
      states { this }
      DynWidget {
        cursor: CursorIcon::Hand,
        on_tap: move |_| this.switch_check(),
        on_key_up: move |k| {
          if k.key == VirtualKeyCode::Space {
            this.switch_check()
          }
        },
        dyns: checkbox
      }
    }
  }
}

impl CustomTheme for CheckBoxTheme {}
#[cfg(test)]
mod tests {
  use crate::prelude::material;

  use super::*;
  use ribir_core::test::{expect_layout_result_with_theme, ExpectRect, LayoutTestItem};

  #[test]
  fn layout() {
    let w = widget! { Checkbox {} };
    expect_layout_result_with_theme(
      w,
      None,
      material::purple::light(),
      &[LayoutTestItem {
        path: &[0],
        expect: ExpectRect {
          x: Some(0.),
          y: Some(0.),
          width: Some(42.),
          height: Some(42.),
        },
      }],
    );
  }

  #[cfg(feature = "png")]
  #[test]
  fn checked_paint() {
    use std::rc::Rc;

    let c = widget! { Checkbox { checked: true } };
    let theme = Rc::new(material::purple::light());
    let mut window = Window::wgpu_headless(c, theme, DeviceSize::new(100, 100));
    window.draw_frame();

    let mut expected = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    expected.push("src/test_imgs/checkbox_checked.png");
    assert!(window.same_as_png(expected));
  }

  #[cfg(feature = "png")]
  #[test]
  fn unchecked_paint() {
    use std::rc::Rc;

    let theme = Rc::new(material::purple::light());
    let mut window =
      Window::wgpu_headless(widget! { Checkbox {} }, theme, DeviceSize::new(100, 100));
    window.draw_frame();
    let mut unchecked_expect = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    unchecked_expect.push("src/test_imgs/checkbox_uncheck.png");
    assert!(window.same_as_png(unchecked_expect));
  }

  #[cfg(feature = "png")]
  #[test]
  fn indeterminate_paint() {
    use std::rc::Rc;

    let c = widget! {
      Checkbox {
        checked: true,
        indeterminate: true,
      }
    };
    let theme = Rc::new(material::purple::light());
    let mut window = Window::wgpu_headless(c.into_widget(), theme, DeviceSize::new(100, 100));
    window.draw_frame();

    let mut expected = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    expected.push("src/test_imgs/checkbox_indeterminate.png");
    assert!(window.same_as_png(expected.clone()));

    let c = widget! {
      Checkbox {
        checked: false,
        indeterminate: true,
      }
    };
    let theme = Rc::new(material::purple::light());
    let mut window = Window::wgpu_headless(c.into_widget(), theme, DeviceSize::new(100, 100));
    window.draw_frame();

    assert!(window.same_as_png(expected));
  }
}
