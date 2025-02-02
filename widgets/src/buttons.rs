use crate::prelude::*;
use ribir_core::prelude::*;

#[derive(Declare)]
pub struct ButtonImpl {
  pub height: f32,
  pub icon_size: Size,
  pub label_gap: f32,
  #[allow(unused)]
  pub icon_pos: IconPosition,
  pub label_style: CowArc<TextStyle>,
  pub foreground_color: Brush,
  #[declare(convert=strip_option)]
  pub background_color: Option<Brush>,
  #[declare(convert=strip_option)]
  pub radius: Option<f32>,
  #[declare(convert=strip_option)]
  pub border_style: Option<Border>,
  #[declare(convert=strip_option)]
  pub padding_style: Option<EdgeInsets>,
}

/// Indicate icon position where before label or after label
#[derive(Default, Clone, Copy)]
pub enum IconPosition {
  #[default]
  Before,
  After,
}

/// Indicate the composition of the internal icon and label of the button
#[derive(Clone, Copy, PartialEq)]
pub enum ButtonType {
  // only icon
  ICON,
  // only label
  LABEL,
  // both icon and label
  BOTH,
}

#[derive(Template)]
pub struct ButtonTemplate {
  label: Option<State<Label>>,
  icon: Option<NamedSvg>,
}

impl ComposeChild for ButtonImpl {
  type Child = ButtonTemplate;

  fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
    let ButtonTemplate { icon, label } = child;
    widget! {
      states { this: this.into_readonly() }
      ConstrainedBox {
        clamp: BoxClamp::fixed_height(this.height),
        DynWidget {
          dyns: BoxDecoration {
            border_radius: this.radius.map(Radius::all),
            background: this.background_color.clone(),
            border: this.border_style.clone(),
          },
          DynWidget {
            dyns: this.padding_style.map(|padding| Padding {
              padding
            }),
            Row {
              v_align: VAlign::Center,
              Option::map(icon, |icon| widget! {
                Icon {
                  size: this.icon_size,
                  DynWidget::from(icon)
                }
              })
              Option::map(label, |label| widget! {
                states { text: label.into_readonly() }
                Margin {
                  margin: EdgeInsets::horizontal(this.label_gap),
                  Text::new(text.0.clone(), &this.foreground_color, this.label_style.clone())
                }
              })
            }
          }
        }
      }
    }
  }
}

mod filled_button;
pub use filled_button::*;

mod outlined_button;
pub use outlined_button::*;

mod button;
pub use button::*;

mod fab_button;
pub use fab_button::*;
