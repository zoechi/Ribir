use crate::{prelude::*, render::render_tree::*, widget::widget_tree::*};
use indextree::*;
use std::collections::{HashMap, HashSet};
mod tree_relationship;
use tree_relationship::Relationship;

#[derive(Default)]
pub struct Application<'a> {
  pub(crate) render_tree: RenderTree,
  pub(crate) widget_tree: WidgetTree<'a>,
  tree_relationship: Relationship,
  /// Store widgets that modified and wait to update its corresponds render
  /// object in render tree.
  dirty_widgets: HashSet<WidgetId>,
  /// Store combination widgets that has require to rebuild its subtree.
  wait_rebuilds: HashSet<WidgetId>,

  dirty_layouts: HashSet<NodeId>,
  dirty_layout_roots: HashSet<NodeId>,
}

impl<'a> Application<'a> {
  #[inline]
  pub fn new() -> Application<'a> { Default::default() }

  pub fn run(mut self, w: Widget<'a>) {
  let root =  self.widget_tree.set_root(w);
    self.widget_tree.inflate(root);
    self.construct_render_tree(
      self.widget_tree.root().expect("widget root should exists"),
    );

    todo!(
      "
      1. update widget tree & render tree when change occurs;
      2. start a event loop to handle event.
      3. run layout and paint for it.
    "
    );

    self.repair_tree();
  }


  /// construct a render tree correspond to widget tree `wid`.
  pub(crate) fn construct_render_tree(&mut self, wid: WidgetId) {
    let (r_wid, rid) = self.widget_render_pair(wid);

    let mut stack = vec![];
    self.render_tree_depth_construct(r_wid, rid, &mut stack);
    while let Some((wid, rid)) = stack.pop() {
      if let Some(sibling) = wid.next_sibling(&self.widget_tree) {
        let (render_widget, render_object) =
          self.append_render_node(sibling, rid);
        stack.push((sibling, rid));
        self.render_tree_depth_construct(
          render_widget,
          render_object,
          &mut stack,
        );
      }
    }
  }

  /// Return a pair of (render widget node id, render object node id) from the
  /// widget node id `wid`, if a render object node not exist, will create it.
  fn widget_render_pair(&mut self, wid: WidgetId) -> (WidgetId, RenderId) {
    let mut r_wid = wid.down_nearest_render_widget(&self.widget_tree);
    if self.render_tree.root().is_none() {
      let render_object = self.create_render_object(r_wid);
      let rid = self.render_tree.set_root(render_object);
      self.tree_relationship.bind(r_wid, rid);
    }

    if let Some(render_id) = self.tree_relationship.widget_to_render(r_wid) {
      (r_wid, *render_id)
    } else {
      let parent = wid
        .parent(&self.widget_tree)
        .expect("should not be a root widget");
      let rw_parent = parent.upper_nearest_render_widget(&self.widget_tree);
      let p_rid = *self.tree_relationship.widget_to_render(rw_parent).expect(
        "parent render object node should construct before construct subtree",
      );
      let (render_widget, render_object) =
        self.append_render_node(r_wid, p_rid);
      r_wid = render_widget;
      (r_wid, render_object)
    }
  }

  fn render_tree_depth_construct(
    &mut self,
    mut wid: WidgetId,
    mut rid: RenderId,
    stack: &mut Vec<(WidgetId, RenderId)>,
  ) {
    wid = wid.down_nearest_render_widget(&self.widget_tree);

    while let Some(w_child_id) = wid.first_child(&self.widget_tree) {
      let (w_child_id, render_object_id) =
        self.append_render_node(w_child_id, rid);
      stack.push((w_child_id, rid));
      rid = render_object_id;
      wid = w_child_id;
    }
  }

  /// Use `wid` to create a render object, and append it into rid.
  /// Return the render widget id which created the render object and the
  /// created render object id.
  fn append_render_node(
    &mut self,
    mut wid: WidgetId,
    rid: RenderId,
  ) -> (WidgetId, RenderId) {
    wid = wid.down_nearest_render_widget(&self.widget_tree);
    let r_child = self.render_tree.new_node(self.create_render_object(wid));
    rid.append(r_child, &mut self.render_tree);
    self.tree_relationship.bind(wid, r_child);
    (wid, r_child)
  }

  fn create_render_object(
    &self,
    render_wid: WidgetId,
  ) -> Box<dyn RenderObjectSafety + Send + Sync> {
    match render_wid.get(&self.widget_tree).expect("must exists!") {
      Widget::Combination(_) => {
        unreachable!("only render widget can create render object!")
      }
      Widget::Render(r) => r.create_render_object(),
      Widget::SingleChild(r) => r.create_render_object(),
      Widget::MultiChild(r) => r.create_render_object(),
    }
  }

  fn repair_tree(&mut self) {
    while let Some(first) = self.wait_rebuilds.iter().nth(0).map(|id| *id) {
      // Always find the topmost widget which need to rebuild to rebuild
      // subtree.
      let top = self.get_rebuild_ancestors(first);
      let widget = top.get_mut(&mut self.widget_tree).expect("Must exist!");

      debug_assert!(
        matches!(widget, Widget::Combination(_)),
        "rebuild widget must be combination widget."
      );

      if let Widget::Combination(ref c) = widget {
        let new_widget = c.build();
        let old_node = top.single_child(&self.widget_tree);
        self.repair_subtree(old_node, new_widget);
        self.wait_rebuilds.remove(&top);
      }
    }
  }

  fn repair_subtree(&mut self, old_node: WidgetId, new_widget: Widget<'a>) {
    let mut stack = vec![(old_node, new_widget)];

    while let Some((old_node, new_widget)) = stack.pop() {
      let old_key = old_node.get(&self.widget_tree).map(|w| w.key()).flatten();
      if old_key.is_some() && old_key == new_widget.key() {
        debug_assert!(new_widget.same_type_widget(
          old_node.get(&self.widget_tree).expect("Must exist!")
        ));
        self.replace_widget(old_node, new_widget, &mut stack)
      } else {
        self.rebuild_subtree(old_node, new_widget);
      }
      self.wait_rebuilds.remove(&old_node);
    }
  }

  /// rebuild the subtree `wid` by the new children `new_children`, the same key
  /// children as before will keep the old subtree and will add into the `stack`
  /// to recursive repair, else will construct a new subtree.
  fn repair_children_by_key(
    &mut self,
    wid: WidgetId,
    new_children: Vec<Widget<'a>>,
    stack: &mut Vec<(WidgetId, Widget<'a>)>,
  ) {
    let mut key_children = HashMap::new();
    let mut child = wid.first_child(&self.widget_tree);
    while let Some(id) = child {
      child = id.next_sibling(&self.widget_tree);
      let key = id
        .get(&self.widget_tree)
        .map(|w| w.key().map(|k| k.clone()))
        .flatten();
      if let Some(key) = key {
        id.detach(&mut self.widget_tree);
        key_children.insert(key, id);
      } else {
        self.drop_subtree(id);
      }
    }

    for w in new_children.into_iter() {
      if let Some(k) = w.key() {
        if let Some(id) = key_children.get(k).map(|id| *id) {
          key_children.remove(k);
          self.replace_widget(id, w, stack);
          continue;
        }
      }

      let child_id = wid.append_widget(w, &mut self.widget_tree);

      self.widget_tree.inflate(child_id);
      self.construct_render_tree(child_id);
    }

    key_children.into_iter().for_each(|(_, v)| {
      self.drop_subtree(v);
    });
  }

  fn replace_widget(
    &mut self,
    old_node: WidgetId,
    mut new_widget: Widget<'a>,
    stack: &mut Vec<(WidgetId, Widget<'a>)>,
  ) {
    match new_widget {
      Widget::Combination(ref c) => {
        let new_child = c.build();
        let old_child_node = old_node.single_child(&self.widget_tree);
        stack.push((old_child_node, new_child));
      }
      Widget::SingleChild(ref mut r) => {
        let new_child = r.take_child();
        let old_child_node = old_node.single_child(&self.widget_tree);
        stack.push((old_child_node, new_child));
      }
      Widget::MultiChild(ref mut multi) => {
        let children = multi.take_children();
        self.repair_children_by_key(old_node, children, stack);
      }
      Widget::Render(_) => {
        // down to leaf, nothing to do.
      }
    }

    *old_node
      .get_mut(&mut self.widget_tree)
      .expect("Old node should exist!") = new_widget;
    self.dirty_widgets.insert(old_node);
  }

  fn rebuild_subtree(&mut self, old_node: WidgetId, new_widget: Widget<'a>) {
    let parent_id = old_node
      .parent(&self.widget_tree)
      .expect("parent should exists!");
    self.drop_subtree(old_node);

    let new_child_id =
      parent_id.append_widget(new_widget, &mut self.widget_tree);

    self.widget_tree.inflate(new_child_id);
    self.construct_render_tree(new_child_id);
  }

  fn drop_subtree(&mut self, wid: WidgetId) {
    let rid = *self
      .tree_relationship
      .widget_to_render(wid.down_nearest_render_widget(&self.widget_tree))
      .expect("must exist");

    let Self {
      widget_tree,
      tree_relationship,
      dirty_widgets,
      wait_rebuilds,
      ..
    } = self;

    wid.descendants(widget_tree).for_each(|id| {
      // clear relationship between render object and render widget.
      if !matches!(id.get(widget_tree), Some(Widget::Combination(_))) {
        tree_relationship.unbind(id)
      }
      dirty_widgets.remove(&id);
      wait_rebuilds.remove(&id);
    });

    // Todo: should remove in a more directly way and not care about
    // relationship
    // Fixme: memory leak here, node not remove.
    wid.detach(&mut self.widget_tree);
    rid.detach(&mut self.render_tree);
  }

  fn get_rebuild_ancestors(&self, wid: WidgetId) -> WidgetId {
    wid
      .ancestors(&self.widget_tree)
      .find(|id| self.wait_rebuilds.contains(id))
      .unwrap_or(wid)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::embed_post::{EmbedPost,create_embed_app};
  extern crate test;
  use test::Bencher;

  #[test]
  fn render_tree_construct() {
    let app = create_embed_app(3);

    assert_eq!(
      app.render_tree.symbol_shape(),
      r#"RowRender { inner_layout: [], size: None }
├── TextRender("Simple demo")
├── TextRender("Adoo")
├── TextRender("Recursive x times")
└── RowRender { inner_layout: [], size: None }
    ├── TextRender("Simple demo")
    ├── TextRender("Adoo")
    ├── TextRender("Recursive x times")
    └── RowRender { inner_layout: [], size: None }
        ├── TextRender("Simple demo")
        ├── TextRender("Adoo")
        ├── TextRender("Recursive x times")
        └── RowRender { inner_layout: [], size: None }
            ├── TextRender("Simple demo")
            ├── TextRender("Adoo")
            └── TextRender("Recursive x times")
"#
    );
  }

  #[test]
  fn drop_subtree() {
    let mut app = create_embed_app(3);
    let id = app.widget_tree.root().unwrap();
    app.drop_subtree(id);

    assert!(app.tree_relationship.is_empty());
    assert!(app.dirty_widgets.is_empty());
    assert!(app.wait_rebuilds.is_empty());

    assert!(app.widget_tree.root().is_none());
    assert!(app.render_tree.root().is_none());
  }

  use crate::test::key_embed_post::KeyDetectEnv;

  fn emit_rebuild(env: &mut KeyDetectEnv) {
    *env.title.borrow_mut() = "New title";
    env
      .app
      .wait_rebuilds
      .insert(env.app.widget_tree.root().unwrap());
  }
  #[test]
  fn repair_tree() {
    let mut env = KeyDetectEnv::new(3);
    env
      .app
      .construct_render_tree(env.app.widget_tree.root().unwrap());
    emit_rebuild(&mut env);

    // fixme: below assert should failed, after support update render tree data.
    assert_eq!(
      env.app.widget_tree.symbol_shape(),
      r#"Combination(EmbedKeyPost { title: RefCell { value: "New title" }, author: "", content: "", level: 3 })
└── MultiChild(KeyDetect { key: KI4(0), child: Row { children: None } })
    ├── Render(KeyDetect { key: KI4(0), child: Text("") })
    ├── Render(KeyDetect { key: KI4(1), child: Text("") })
    ├── Render(KeyDetect { key: KI4(2), child: Text("") })
    └── Combination(KeyDetect { key: KString("embed"), child: EmbedKeyPost { title: RefCell { value: "New title" }, author: "", content: "", level: 2 } })
        └── MultiChild(KeyDetect { key: KI4(0), child: Row { children: None } })
            ├── Render(KeyDetect { key: KI4(0), child: Text("") })
            ├── Render(KeyDetect { key: KI4(1), child: Text("") })
            ├── Render(KeyDetect { key: KI4(2), child: Text("") })
            └── Combination(KeyDetect { key: KString("embed"), child: EmbedKeyPost { title: RefCell { value: "New title" }, author: "", content: "", level: 1 } })
                └── MultiChild(KeyDetect { key: KI4(0), child: Row { children: None } })
                    ├── Render(KeyDetect { key: KI4(0), child: Text("") })
                    ├── Render(KeyDetect { key: KI4(1), child: Text("") })
                    ├── Render(KeyDetect { key: KI4(2), child: Text("") })
                    └── Combination(KeyDetect { key: KString("embed"), child: EmbedKeyPost { title: RefCell { value: "New title" }, author: "", content: "", level: 0 } })
                        └── MultiChild(KeyDetect { key: KI4(0), child: Row { children: None } })
                            ├── Render(KeyDetect { key: KI4(0), child: Text("") })
                            ├── Render(KeyDetect { key: KI4(1), child: Text("") })
                            └── Render(KeyDetect { key: KI4(2), child: Text("") })
"#
    );

    // fixme: below assert should failed, after support update render tree.
    assert_eq!(
      env.app.render_tree.symbol_shape(),
      r#"KeyRender(RowRender { inner_layout: [], size: None })
├── KeyRender(TextRender(""))
├── KeyRender(TextRender(""))
├── KeyRender(TextRender(""))
└── KeyRender(RowRender { inner_layout: [], size: None })
    ├── KeyRender(TextRender(""))
    ├── KeyRender(TextRender(""))
    ├── KeyRender(TextRender(""))
    └── KeyRender(RowRender { inner_layout: [], size: None })
        ├── KeyRender(TextRender(""))
        ├── KeyRender(TextRender(""))
        ├── KeyRender(TextRender(""))
        └── KeyRender(RowRender { inner_layout: [], size: None })
            ├── KeyRender(TextRender(""))
            ├── KeyRender(TextRender(""))
            └── KeyRender(TextRender(""))
"#
    );
  }

  // fn assert_root_bound(app: &mut Application, bound: Option<Size>) {
  //   let root = app.r_arena.get_mut(app.render_tree.unwrap()).unwrap();
  //   let render_box = root.get_mut().to_render_box().unwrap();
  //   assert_eq!(render_box.bound(), bound);
  // }

  // fn layout_app(app: &mut Application) {
  //   let mut_ptr = &mut app.r_arena as *mut Arena<Box<(dyn RenderObject + Send
  // + Sync)>>;   let mut ctx = RenderCtx::new(&mut app.r_arena, &mut
  // app.dirty_layouts, &mut app.dirty_layout_roots);   unsafe {
  //       let root =
  // mut_ptr.as_mut().unwrap().get_mut(app.render_tree.unwrap()).unwrap();
  //       root.get_mut().perform_layout(app.render_tree.unwrap(), &mut ctx);
  //   }
  // }

  // fn mark_dirty(app: &mut Application, node_id: NodeId) {
  //   let mut_ptr = &mut app.r_arena as *mut Arena<Box<(dyn RenderObject + Send
  // + Sync)>>;   let mut ctx = RenderCtx::new(&mut app.r_arena, &mut
  // app.dirty_layouts, &mut app.dirty_layout_roots);

  //   unsafe {
  //      mut_ptr
  //       .as_mut()
  //       .unwrap()
  //       .get_mut(node_id)
  //       .unwrap()
  //       .get_mut()
  //       .mark_dirty(node_id, &mut ctx);
  //   }
  // }

  // #[test]
  // fn test_layout() {
  //   let post = EmbedPost {
  //     title: "Simple demo",
  //     author: "Adoo",
  //     content: "Recursive 5 times",
  //     level: 5,
  //   };
  //   let mut app = Application::new();
  //   app.inflate(post.to_widget());
  //   app.construct_render_tree(app.widget_tree.unwrap());

  //   let root_id = app.render_tree.unwrap();
  //   mark_dirty(&mut app, root_id);
  //   layout_app(&mut app);
  //   assert_root_bound(
  //     &mut app,
  //     Some(Size {
  //       width: 192,
  //       height: 1,
  //     }),
  //   );

  //   let last_child_id = app
  //     .r_arena
  //     .get(app.render_tree.unwrap())
  //     .unwrap()
  //     .last_child()
  //     .unwrap();
  //   mark_dirty(&mut app, last_child_id);
  //   assert_eq!(app.dirty_layouts.contains(&root_id), true);

  //   layout_app(&mut app);
  //   assert_eq!(app.dirty_layouts.contains(&root_id), false);
  //   assert_root_bound(
  //     &mut app,
  //     Some(Size {
  //       width: 192,
  //       height: 1,
  //     }),
  //   );
  // }

  #[bench]
  fn repair_5_x_1000(b: &mut Bencher) {
    let mut env = KeyDetectEnv::new(1000);
    env
      .app
      .construct_render_tree(env.app.widget_tree.root().unwrap());
    b.iter(|| {
      emit_rebuild(&mut env);
      env.app.repair_tree();
    });
  }

  #[bench]
  fn render_tree_5_x_1000(b: &mut Bencher) {
    b.iter(|| {
       create_embed_app(1000)
    });
  }
}
