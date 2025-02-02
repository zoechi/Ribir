use super::{flex::*, Direction};
use ribir_core::prelude::*;

#[derive(Default, Declare, Clone)]
pub struct Row {
  #[declare(default)]
  pub reverse: bool,
  #[declare(default)]
  pub wrap: bool,
  #[declare(default = Align::Center)]
  pub align_items: Align,
  #[declare(default)]
  pub justify_content: JustifyContent,
}

impl ComposeChild for Row {
  type Child = Vec<Widget>;
  fn compose_child(this: State<Self>, children: Self::Child) -> Widget {
    widget! {
      states { this: this.into_readonly() }
      Flex {
        reverse: this.reverse,
        wrap: this.wrap,
        direction: Direction::Horizontal,
        align_items: this.align_items,
        justify_content: this.justify_content,
        DynWidget { dyns: children }
      }
    }
  }
}
