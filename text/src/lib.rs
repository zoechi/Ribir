//! A tiny low level text processing library dedicate to Ribir, use to reorder,
//! shape and do simple layout for text. It's focus
//!
//! Some detail processing learn from [usvg](https://github.com/RazrFalcon/resvg/blob/master/usvg/src/text)
#![feature(test)]
pub mod font_db;
pub mod shaper;
use derive_more::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use fontdb::ID;
pub use fontdb::{Stretch as FontStretch, Style as FontStyle, Weight as FontWeight};
pub use ribir_algo::Substr;
use rustybuzz::ttf_parser::GlyphId;
use std::hash::Hash;
pub mod text_reorder;
pub mod typography;
use ordered_float::OrderedFloat;
pub use text_reorder::TextReorder;
mod typography_store;
pub use typography_store::{TypographyStore, VisualGlyphs};

mod text_writer;
pub use text_writer::{CharacterCursor, ControlChar, TextWriter};

mod grapheme_cursor;
pub use grapheme_cursor::GraphemeCursor;

pub mod unicode_help;

/// Unit for convert between pixel and em.
pub const PIXELS_PER_EM: f32 = 16.;

/// `Pixels is an absolute length unit and relative to the view device
#[derive(
  Debug,
  Default,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Add,
  Sub,
  Div,
  AddAssign,
  Mul,
  SubAssign,
  Eq,
  Ord,
  Hash,
)]
pub struct Pixel(pub OrderedFloat<f32>);

///  `Em` is relative length unit relative to `Pixel`. We stipulate Em(1.) equal
/// to Pixel(16.)
#[derive(
  Debug,
  Default,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Add,
  Sub,
  Div,
  AddAssign,
  Mul,
  SubAssign,
  Eq,
  Ord,
  Hash,
)]
pub struct Em(OrderedFloat<f32>);

/// The size of font. `Pixels is an absolute length unit and relative to the
/// view device, and `Em` is relative length unit relative to `Pixel`. We
/// stipulate FontSize::Em(1.) equal to FontSize::Pixel(16.)
#[derive(Debug, Clone, Copy, Add)]
pub enum FontSize {
  Pixel(Pixel),
  Em(Em),
}

// Enum value descriptions are from the CSS spec.
/// A [font family](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#propdef-font-family).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontFamily {
  // todo: no need cow? or directly face ids
  /// The name of a font family of choice.
  Name(std::borrow::Cow<'static, str>),

  /// Serif fonts represent the formal text style for a script.
  Serif,

  /// Glyphs in sans-serif fonts, as the term is used in CSS, are generally low
  /// contrast and have stroke endings that are plain — without any flaring,
  /// cross stroke, or other ornamentation.
  SansSerif,

  /// Glyphs in cursive fonts generally use a more informal script style,
  /// and the result looks more like handwritten pen or brush writing than
  /// printed letterwork.
  Cursive,

  /// Fantasy fonts are primarily decorative or expressive fonts that
  /// contain decorative or expressive representations of characters.
  Fantasy,

  /// The sole criterion of a monospace font is that all glyphs have the same
  /// fixed width.
  Monospace,
}

/// Encapsulates the font properties of font face.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontFace {
  /// A prioritized list of font family names or generic family names.
  ///
  /// [font-family](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#propdef-font-family) in CSS.
  pub families: Box<[FontFamily]>,
  /// Selects a normal, condensed, or expanded face from a font family.
  ///
  /// [font-stretch](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-stretch-prop) in CSS.
  pub stretch: FontStretch,
  /// Allows italic or oblique faces to be selected.
  ///
  /// [font-style](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-style-prop) in CSS.
  pub style: FontStyle,
  /// Specifies the weight of glyphs in the font, their degree of blackness or
  /// stroke thickness.
  ///
  /// [font-weight](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-weight-prop) in CSS.
  pub weight: FontWeight,
}

#[derive(Debug, Clone)]
pub struct Glyph<Unit> {
  /// The font face id of the glyph.
  pub face_id: ID,
  /// How many units the line advances after drawing this glyph when setting
  /// text in horizontal direction.
  pub x_advance: Unit,
  /// How many units the line advances after drawing this glyph when setting
  /// text in vertical direction.
  pub y_advance: Unit,
  /// How many units the glyph moves on the X-axis before drawing it, this
  /// should not affect how many the line advances.
  pub x_offset: Unit,
  /// How many units the glyph moves on the Y-axis before drawing it, this
  /// should not affect how many the line advances.
  pub y_offset: Unit,
  /// The id of the glyph.
  pub glyph_id: GlyphId,
  /// An cluster of origin text as byte index.
  pub cluster: u32,
}

impl Default for FontFace {
  fn default() -> Self {
    Self {
      families: Box::new([FontFamily::Serif]),
      stretch: Default::default(),
      style: Default::default(),
      weight: Default::default(),
    }
  }
}

/// Text-align relative to the horizontal or vertical, not caring about whether
/// the text is left-to-right or right-to-left, In the horizontal the left is
/// the start, and in vertical the top is the start.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextAlign {
  Start,
  Center,
  End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextDirection {
  /// Text is set horizontally from left to right.
  LeftToRight,
  /// Text is set horizontally from right to left.
  RightToLeft,
  /// Text is set vertically from top to bottom.
  TopToBottom,
  /// Text is set vertically from bottom to top.
  BottomToTop,
}

impl TextDirection {
  #[inline]
  pub fn is_vertical(&self) -> bool {
    matches!(
      self,
      TextDirection::TopToBottom | TextDirection::BottomToTop
    )
  }

  #[inline]
  pub fn is_horizontal(&self) -> bool {
    matches!(
      self,
      TextDirection::LeftToRight | TextDirection::RightToLeft
    )
  }
}

impl From<f32> for Pixel {
  #[inline]
  fn from(v: f32) -> Self { Pixel(v.into()) }
}

impl From<Pixel> for Em {
  #[inline]
  fn from(p: Pixel) -> Self { Em(p.0 / PIXELS_PER_EM) }
}

impl From<Em> for Pixel {
  #[inline]
  fn from(e: Em) -> Self { Pixel(e.0 * PIXELS_PER_EM) }
}

impl PartialEq<Em> for Pixel {
  #[inline]
  fn eq(&self, other: &Em) -> bool {
    let p: Pixel = (*other).into();
    *self == p
  }
}

impl PartialEq<Pixel> for Em {
  #[inline]
  fn eq(&self, other: &Pixel) -> bool {
    let p: Pixel = (*self).into();
    p == *other
  }
}

impl FontSize {
  #[inline]
  pub fn into_pixel(self) -> Pixel {
    match self {
      FontSize::Pixel(p) => p,
      FontSize::Em(e) => e.into(),
    }
  }

  #[inline]
  pub fn into_em(self) -> Em {
    match self {
      FontSize::Pixel(p) => p.into(),
      FontSize::Em(e) => e,
    }
  }

  /// Em scale by font size.
  #[inline]
  pub fn relative_em(self, em: f32) -> Em { self.into_em() * em }
}

impl PartialEq for FontSize {
  #[inline]
  fn eq(&self, other: &Self) -> bool { self.into_pixel() == other.into_pixel() }
}

impl lyon_path::geom::euclid::num::Zero for Em {
  #[inline]
  fn zero() -> Self { Em(f32::zero().into()) }
}

impl lyon_path::geom::euclid::num::Zero for Pixel {
  #[inline]
  fn zero() -> Self { Pixel(f32::zero().into()) }
}

impl Em {
  pub const MAX: Em = Em(OrderedFloat(f32::MAX));

  #[inline]
  pub fn value(self) -> f32 { self.0.into() }

  #[inline]
  pub fn absolute(em: f32) -> Self { Self(em.into()) }

  #[inline]
  pub fn relative_to(em: f32, font_size: FontSize) -> Self { font_size.relative_em(em) }
}

impl Pixel {
  #[inline]
  pub fn value(self) -> f32 { *self.0 }
}

impl std::ops::Mul<Em> for Em {
  type Output = Em;
  #[inline]
  fn mul(self, rhs: Em) -> Self::Output { Em(self.0 * rhs.0) }
}

impl std::ops::Mul<Pixel> for Pixel {
  type Output = Pixel;
  #[inline]
  fn mul(self, rhs: Pixel) -> Self::Output { Pixel(self.0 * rhs.0) }
}

impl std::ops::MulAssign<f32> for Em {
  #[inline]
  fn mul_assign(&mut self, rhs: f32) { self.0 *= rhs; }
}

impl std::ops::Div<Em> for Em {
  type Output = Em;

  #[inline]
  fn div(self, rhs: Em) -> Self::Output { Em(self.0 / rhs.0) }
}

impl std::ops::Div<Pixel> for Pixel {
  type Output = Pixel;

  #[inline]
  fn div(self, rhs: Pixel) -> Self::Output { Pixel(self.0 / rhs.0) }
}

impl<U> Glyph<U> {
  pub fn cast<T>(self) -> Glyph<T>
  where
    U: Into<T>,
  {
    let Glyph {
      face_id,
      x_advance,
      y_advance,
      x_offset,
      y_offset,
      glyph_id,
      cluster,
    } = self;

    Glyph {
      face_id,
      x_advance: x_advance.into(),
      y_advance: y_advance.into(),
      x_offset: x_offset.into(),
      y_offset: y_offset.into(),
      glyph_id,
      cluster,
    }
  }
}
