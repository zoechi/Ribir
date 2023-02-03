use lyon_algorithms::geom::{QuadraticBezierSegment, CubicBezierSegment};
use ribir::prelude::shaper::TextShaper;
use ribir::prelude::{*, font_db::FontDB};
use lyon_path::{math::Point, Event};
use std::ops::Add;
use std::sync::{Arc, RwLock};
use std::time::Duration;

fn find_nearest_point_pair_from_two_points(
  from_list: &Vec<[Point; 4]>,
  to_list: &Vec<[Point; 4]>,
) -> Vec<(usize, usize)> {
  let mut pair_result = vec![];
  let f_len = from_list.len();
  let t_len = to_list.len();

  if f_len > t_len {
    for (t_idx, tp) in from_list.iter().enumerate() {
  
      let mut find_min_idx = 0;
      let mut min_distance_count = f32::INFINITY;
  
      let tp0 = &tp[0];
      let tp3 = &tp[3];
  
      for (f_idx, fp) in to_list.iter().enumerate() {
        let fp0 = &fp[0];
        let fp3 = &fp[3];
  
        let d0 = fp0.distance_to(tp0.clone());
        let d3 = fp3.distance_to(tp3.clone());
  
        let distance_count = d0 + d3;
  
        if distance_count < min_distance_count {
          min_distance_count = distance_count;
          find_min_idx = f_idx;
        }
      }
  
      pair_result.push((t_idx, find_min_idx));
    }
  } else {
    for (t_idx, tp) in to_list.iter().enumerate() {
  
      let mut find_min_idx = 0;
      let mut min_distance_count = f32::INFINITY;
  
      let tp0 = &tp[0];
      let tp3 = &tp[3];
  
      for (f_idx, fp) in from_list.iter().enumerate() {
        let fp0 = &fp[0];
        let fp3 = &fp[3];
  
        let d0 = fp0.distance_to(tp0.clone());
        let d3 = fp3.distance_to(tp3.clone());
  
        let distance_count = d0 + d3;
  
        if distance_count < min_distance_count {
          min_distance_count = distance_count;
          find_min_idx = f_idx;
        }
      }
  
      pair_result.push((find_min_idx, t_idx));
    }
  }

  pair_result
}

fn get_text_path<T: Into<Substr>>(typography_store: &TypographyStore, text: T, style: &TextStyle) -> Vec<Path> {
  let visual_glyphs = typography_with_text_style(typography_store, text, style, None);
    let glyphs = visual_glyphs.pixel_glyphs().collect::<Vec<_>>();
    glyphs.into_iter().map(|g| {
      let Glyph { glyph_id, face_id, x_offset, y_offset, .. } = g;
      let face = {
        let mut font_db = typography_store.shaper.font_db_mut();
        font_db
          .face_data_or_insert(face_id)
          .expect("Font face not exist!")
          .clone()
      };
      let font_size_ems = style.font_size.into_pixel().value();
      let t = euclid::Transform2D::default()
        .pre_translate((x_offset.value(), y_offset.value()).into())
        .pre_scale(font_size_ems, font_size_ems);
      Path {
        path: face.outline_glyph(glyph_id).unwrap().transformed(&t),
        style: PathStyle::Stroke(StrokeOptions::default()),
        // style: PathStyle::Fill,
      }
    }).collect::<Vec<_>>()
}

fn text_path_lerp_fn() -> impl Fn(&Vec<PathPaintKit>, &Vec<PathPaintKit>, f32) -> Vec<PathPaintKit> + Clone
{
  move |from, to, rate| {
    let from_size = from.len();
    let second_size = to.len();
    let mut result = vec![];

    if from_size > second_size {
      todo!();
    } else if from_size < second_size {
      todo!();
    } else {
      for (idx, path_paint_kit) in from.into_iter().enumerate() {
        let from_path = path_paint_kit.path.path.iter();
        let mut from_path_points = vec![];

        for path_paint_kit in from_path {
          match path_paint_kit {
            Event::Begin { at } => {},
            Event::Line { from, to } => {
              from_path_points.push([from, from, to, to]);
            },
            Event::Quadratic { from, ctrl, to } => {
              let seg = QuadraticBezierSegment { from, ctrl, to }.to_cubic();
              from_path_points.push([seg.from, seg.ctrl1, seg.ctrl2, seg.to]);
            },
            Event::Cubic { from, ctrl1, ctrl2, to } => {
              from_path_points.push([from, ctrl1, ctrl2, to]);
            },
            Event::End { last, first, close } => {},
          }
        }

        let to_path = to.get(idx).unwrap().path.path.iter();
        let mut to_path_points = vec![];

        for path_paint_kit in to_path {
          match path_paint_kit {
            Event::Begin { at } => {},
            Event::Line { from, to } => {
              to_path_points.push([from, from, to, to]);
            },
            Event::Quadratic { from, ctrl, to } => {
              let seg = QuadraticBezierSegment { from, ctrl, to }.to_cubic();
              to_path_points.push([seg.from, seg.ctrl1, seg.ctrl2, seg.to]);
            },
            Event::Cubic { from, ctrl1, ctrl2, to } => {
              to_path_points.push([from, ctrl1, ctrl2, to]);
            },
            Event::End { last, first, close } => {},
          }
        }

        let pair = find_nearest_point_pair_from_two_points(&from_path_points, &to_path_points);
        let last_idx = pair.len() - 1;
        let mut result_path = lyon_path::Path::builder();
        for (idx, (from_idx, to_idx)) in pair.into_iter().enumerate() {
          let from_points = from_path_points.get(from_idx).unwrap();
          let to_points = to_path_points.get(to_idx).unwrap();
          let ctrl1 = from_points[1].add((to_points[1] - from_points[1]) * rate);
          let ctrl2 = from_points[2].add((to_points[2] - from_points[2]) * rate);
          let to = from_points[3].add((to_points[2] - from_points[2]) * rate);

          if idx == 0 {
            let from = from_points[0].add((to_points[0] - from_points[0]) * rate);
            result_path.begin(from);
          }

          result_path.cubic_bezier_to(ctrl1, ctrl2, to);

          if idx == last_idx {
            result_path.end(true);
          }
        }

        let path = result_path.build();

        result.push(PathPaintKit {
          path: Path {
            path,
            style: PathStyle::Stroke(StrokeOptions::default()),
            // style: PathStyle::Fill,
          },
          brush: Brush::Color(Color::BLACK),
        });
      }
    }
    
    result
  }
}

fn main() {
  let mut font_db = FontDB::default();
  font_db.load_system_fonts();
  let font_db = Arc::new(RwLock::new(font_db));
  let shaper = TextShaper::new(font_db.clone());
  let reorder = TextReorder::default();
  let typography_store = TypographyStore::new(reorder.clone(), font_db, shaper.clone());
  let w = widget! {
    init ctx => {
      let text_style = TypographyTheme::of(ctx).headline1.text.clone();
      let init_path = get_text_path(&typography_store, "2", &text_style);
      let init_paint_path_kit = init_path.into_iter().map(|path| {
        PathPaintKit {
          path,
          brush: Brush::Color(Color::BLACK),
        }
      }).collect::<Vec<_>>();
      let finally_path = get_text_path(&typography_store, "1", &text_style);
      let finally_paint_path_kit = finally_path.into_iter().map(|path| {
        PathPaintKit {
          path,
          brush: Brush::Color(Color::BLACK),
        }
      }).collect::<Vec<_>>();
    }

    PathsPaintKit {
      id: path_kit,
      paths: finally_paint_path_kit,
      mounted: move |_| {
        animate.run();
      }
    }

    Animate {
      id: animate,
      transition: Transition {
        delay: Some(Duration::from_millis(1500)),
        duration: Duration::from_millis(5000),
        easing: easing::LINEAR,
        repeat: None,
      },
      prop: prop!(path_kit.paths, text_path_lerp_fn()),
      from: init_paint_path_kit,
    }

  };

  app::run(w);
}
