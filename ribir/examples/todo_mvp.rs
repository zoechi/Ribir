use ribir::prelude::{*, svgs::CLOSE};

#[derive(Debug, Clone, PartialEq)]
struct Task {
  finished: bool,
  label: String,
}
#[derive(Debug)]
struct TodoMVP {
  tasks: Vec<Task>,
}

impl Compose for TodoMVP {
  fn compose(this: StateWidget<Self>) -> Widget {
    widget! {
      // split this to avoid mutable borrow conflict in `DynWidget`.
      track {
        this: this.into_stateful(),
      }
      Column {
        margin: EdgeInsets::all(10.),
        Row {
          margin: EdgeInsets::only_bottom(10.),
          SizedBox {
            size: Size::new(240., 30.),
            border: Border::only_bottom(BorderSide { width:1., color: Color::BLACK }),
            Input {
              id: input,
              placeholder: String::from("Add Task"),
            }
          }
          SizedBox {
            size: Size::new(60., 30.),
            margin: EdgeInsets::only_left(20.),
            border_radius: Radius::all(4.),
            border: Border::all(BorderSide { width: 1., color: Color::BLACK }),
            tap: move |_| {
              this.tasks.push(Task {
                label: input.text_in_show(),
                finished: false,
              });
              input.text = String::default();
            },
            Row {
              Icon { svgs::ADD }
              Text {
                text: "Add",
                style: TypographyTheme::of(ctx).button.text.clone(),
              }
            }
          }
        }

        Tabs {
          id: tabs,
          Tab {
            TabHeader {
              TabText {
                tab_text: String::from("All"),
                is_active: tabs.cur_idx == 0,
              }
            }
            TabPane {
              DynWidget {
                dyns: TodoMVP::pane(this.clone_stateful(), |_| true)
              }
            }
          }
          Tab {
            TabHeader {
              TabText {
                tab_text: String::from("Active"),
                is_active: tabs.cur_idx == 1,
              }
            }
            TabPane {
              DynWidget {
                dyns: TodoMVP::pane(this.clone_stateful(), |task| !task.finished)
              }
            }
          }
          Tab {
            TabHeader {
              TabText {
                tab_text: String::from("Completed"),
                is_active: tabs.cur_idx == 2,
              }
            }
            TabPane {
              DynWidget {
                dyns: TodoMVP::pane(this.clone_stateful(), |task| task.finished)
              }
            }
          }
        }
      }
    }
  }
}

impl TodoMVP {
  fn pane(this: Stateful<Self>, cond: impl Fn(&Task) -> bool + 'static) -> Widget {
    widget! {
      track { this, this2: this.clone() }
      VScrollBar {
        background: Brush::Color(Color::BURLYWOOD),
        Lists {
          // align_items: Align::Start,
          padding: EdgeInsets::all(8.),
          ListItem {
            HeadlineText(String::from("text"))
            SupportingText(String::from("lll"))
            Leading {
              Icon {
                svgs::CLOSE
              }
            }
          }
        }
      }
    }
  }
}

#[derive(Debug, Declare)]
struct TabText {
  is_active: bool,
  tab_text: String,
}

impl Compose for TabText {
  fn compose(this: StateWidget<Self>) -> Widget {
    widget! {
      track {
        this: this.into_stateful()
      }
      Text {
        text: this.tab_text.clone(),
        padding: EdgeInsets::all(4.),
        h_align: HAlign::Center,
        style: TextStyle {
          foreground: if this.is_active { Brush::Color(Color::RED) } else { Brush::Color(Color::BLACK) },
          ..Default::default()
        },
      }
    }
  }
}

fn main() {
  env_logger::init();

  let todo = TodoMVP {
    tasks: vec![
      Task {
        finished: true,
        label: "Implement Checkbox".to_string(),
      },
      Task {
        finished: true,
        label: "Support Scroll".to_string(),
      },
      Task {
        finished: false,
        label: "Support Virtual Scroll".to_string(),
      },
      Task {
        finished: false,
        label: "Support data bind".to_string(),
      },
    ],
  }
  .into_stateful();

  app::run(todo.into_widget());
}
