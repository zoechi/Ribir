use lyon_algorithms::geom::QuadraticBezierSegment;
use ribir::prelude::{*, font_db::FontDB, shaper::TextShaper};
use std::{sync::{Arc, RwLock}};
use lyon_path::{math::{Point, Vector}, Event};

fn main() {
  let mut font_db = FontDB::default();
  font_db.load_system_fonts();
  let font_db = Arc::new(RwLock::new(font_db));
  let shaper = TextShaper::new(font_db.clone());
  let reorder = TextReorder::default();
  let typography_store = TypographyStore::new(reorder.clone(), font_db, shaper.clone());
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CubicSegment {
  // cubic segment end point
  end_point: Point,
  // cubic segment two control point c1, c2
  ctrl_points: Option<(Point, Point)>,
}

fn get_points_from_path(path: &lyon_path::Path) -> Vec<Vec<CubicSegment>> {
  let mut multi_paths = vec![];
  for evt in path.iter() {
    match evt {
      Event::Begin { at } => {
        let mut cur = vec![];
        cur.push(CubicSegment { end_point: at, ctrl_points: None });
        multi_paths.push(cur);
      },
      Event::Line { from: _, to } => {
        if let Some(cur) = multi_paths.last_mut() {
          cur.push(CubicSegment { end_point: to, ctrl_points: Some((to, to)) });
        } else {
          unreachable!("Path must be start with Event::Begin!");
        }
      },
      Event::Quadratic { from, ctrl, to } => {
        if let Some(cur) = multi_paths.last_mut() {
          let seg = QuadraticBezierSegment { from, ctrl, to }.to_cubic();
          cur.push(CubicSegment { end_point: seg.to, ctrl_points: Some((seg.ctrl1, seg.ctrl2)) });
        } else {
          unreachable!("Path must be start with Event::Begin!");
        }
      },
      Event::Cubic { from: _, ctrl1, ctrl2, to } => {
        if let Some(cur) = multi_paths.last_mut() {
          cur.push(CubicSegment { end_point: to, ctrl_points: Some((ctrl1, ctrl2)) });
        } else {
          unreachable!("Path must be start with Event::Begin!");
        }
      },
      Event::End { last: _, first: _, close: _ } => {},
    }
  }

  multi_paths
}

/// Get the character path through text and text_style
fn get_text_paths<T: Into<Substr>>(
  typography_store: &TypographyStore,
  text: T,
  style: &TextStyle,
) -> Vec<Path> {
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

/// compare two path points vector info to find nearest point pair
fn find_nearest_point_pair(
  source: &Vec<Vector>,
  target: &Vec<Vector>,
) -> Vec<(usize, usize)> {
  todo!()
}

fn find_nearest_path_pair(
  source: &Vec<Vec<Vector>>,
  target: &Vec<Vec<Vector>>,
) -> Vec<(Option<usize>, Option<usize>)> {
  todo!()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn path_single_move_to_cubic_point() {
    let mut path = lyon_path::Path::builder();
    path.begin(Point::new(25., 25.));
    path.line_to(Point::new(30., 30.));
    path.line_to(Point::new(45., 60.));
    path.line_to(Point::new(25., 80.));
    path.line_to(Point::new(25., 25.));
    path.end(true);

    let path = path.build();
    let points = get_points_from_path(&path);
    let expect_result = vec![
      vec![
        CubicSegment { end_point: Point::new(25., 25.), ctrl_points: None },
        CubicSegment { end_point: Point::new(30., 30.), ctrl_points: Some((Point::new(30., 30.), Point::new(30., 30.))) },
        CubicSegment { end_point: Point::new(45., 60.), ctrl_points: Some((Point::new(45., 60.), Point::new(45., 60.))) },
        CubicSegment { end_point: Point::new(25., 80.), ctrl_points: Some((Point::new(25., 80.), Point::new(25., 80.))) },
        CubicSegment { end_point: Point::new(25., 25.), ctrl_points: Some((Point::new(25., 25.), Point::new(25., 25.))) },
      ]
    ];

    assert_eq!(points, expect_result);
  }

  #[test]
  fn path_multi_move_to_cubic_point() {
    let mut path = lyon_path::Path::builder(); path.begin(Point::new(25., 25.));
    path.line_to(Point::new(30., 30.));
    path.line_to(Point::new(45., 60.));
    path.line_to(Point::new(25., 80.));
    path.line_to(Point::new(25., 25.));
    path.end(false);
    path.begin(Point::new(45., 45.));
    path.line_to(Point::new(60., 60.));
    path.line_to(Point::new(80., 20.));
    path.line_to(Point::new(45., 45.));
    path.end(true);
    
    let path = path.build();
    let points = get_points_from_path(&path);
    let expect_result = vec![
      vec![
        CubicSegment { end_point: Point::new(25., 25.), ctrl_points: None },
        CubicSegment { end_point: Point::new(30., 30.), ctrl_points: Some((Point::new(30., 30.), Point::new(30., 30.))) },
        CubicSegment { end_point: Point::new(45., 60.), ctrl_points: Some((Point::new(45., 60.), Point::new(45., 60.))) },
        CubicSegment { end_point: Point::new(25., 80.), ctrl_points: Some((Point::new(25., 80.), Point::new(25., 80.))) },
        CubicSegment { end_point: Point::new(25., 25.), ctrl_points: Some((Point::new(25., 25.), Point::new(25., 25.))) },
      ],
      vec![
        CubicSegment { end_point: Point::new(45., 45.), ctrl_points: None },
        CubicSegment { end_point: Point::new(60., 60.), ctrl_points: Some((Point::new(60., 60.), Point::new(60., 60.))) },
        CubicSegment { end_point: Point::new(80., 20.), ctrl_points: Some((Point::new(80., 20.), Point::new(80., 20.))) },
        CubicSegment { end_point: Point::new(45., 45.), ctrl_points: Some((Point::new(45., 45.), Point::new(45., 45.))) },
      ]
    ];

    assert_eq!(points, expect_result);
  }
}