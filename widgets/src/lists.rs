use crate::prelude::*;
use ribir_core::{prelude::*};

#[derive(Declare, Default)]
pub struct Lists {
  #[declare(default = false)]
  divider: bool,
}

#[derive(Default, Declare, SingleChild)]
pub struct Leading;

#[derive(Default, Declare, SingleChild)]
pub struct Trailing;

#[derive(Clone, PartialEq)]
pub enum EdgePosition {
  Frist,
  Last,
  None,
}

#[derive(Clone, Declare)]
pub struct ListItemStyle {
  #[declare(default = false)]
  pub divider: bool,
  pub edge: EdgePosition,
}

impl ComposeStyle for ListItemStyle {
  type Host = Widget;
  #[inline]
  fn compose_style(_this: Stateful<Self>, host: Self::Host) -> Widget
  where
    Self: Sized,
  {
    host
  }
}

pub struct HeadlineText(pub String);
pub struct SupportingText(pub String);

#[derive(Default, Declare)]
pub struct ListItem;

#[derive(Template)]
pub struct ListItemTemplate {
  headline_text: HeadlineText,
  supporting_text: Option<SupportingText>,
  leading: Option<WidgetOf<Leading>>,
  trailing: Option<WidgetOf<Trailing>>,
}

impl ComposeChild for ListItem {
  type Child = ListItemTemplate;

  fn compose_child(_this: StateWidget<Self>, child: Self::Child) -> Widget {
    let ListItemTemplate { headline_text, supporting_text, leading, trailing } = child;

    widget! {
      Row {
        DynWidget {
          dyns: leading.map(|w| w.child)
        }
        Expanded {
          flex: 1.,
          
          Column {
            border: Border::only_bottom(BorderSide { width:1., color: Palette::of(ctx).primary() }),
            DynWidget {
              dyns: widget! {
                Text {
                  text: headline_text.0
                }
              }
            }
            DynWidget {
              dyns: supporting_text.map(|text| {
                widget! {
                  Text {
                    text: text.0
                  }
                }
              })
            }
          }
        }
        DynWidget {
          dyns: trailing.map(|w| w.child)
        }
      }
    }
  }

}

impl ComposeChild for Lists {
  type Child = Vec<ListItem>;

  fn compose_child(this: StateWidget<Self>, children: Self::Child) -> Widget {
    let last_idx = children.len() - 1;

    widget! {
      track {
        this: this.into_stateful()
      }
      Column {
        DynWidget {
          dyns: children.into_iter().enumerate().map(move |(idx, w)| {
            let edge = if idx == 0 {
              EdgePosition::Frist
            } else if idx == last_idx {
              EdgePosition::Last
            } else {
              EdgePosition::None
            };

            widget! {
              ListItemStyle {
                divider: this.divider,
                edge,

                DynWidget {
                  dyns: w
                }
              }
            }
          })
        }
      }
    }
  }
}
