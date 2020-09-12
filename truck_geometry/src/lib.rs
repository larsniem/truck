//! # Overview
//! `truck_geometry` is a crate for describing geometrical information.
//! It contains definision basic mathematical objects, vectors and matrices.

#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

extern crate cgmath;
extern crate serde;
use std::fmt::Debug;

use serde::{Serialize, Deserialize};
pub use cgmath::prelude::*;

macro_rules! f64_type {
    ($typename: ident) => {
        /// redefinition, scalar = f64
        pub type $typename = cgmath::$typename<f64>;
    };
    ($a: ident, $($b: ident), *) => { f64_type!($a); f64_type!($($b),*); }
}
f64_type!(Vector1, Vector2, Vector3, Vector4, Matrix2, Matrix3, Matrix4, Point1, Point2, Point3);

/// knot vector
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct KnotVec(Vec<f64>);

/// bounding box
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BoundingBox<V>(V, V);

/// general tolerance
pub const TOLERANCE: f64 = 1.0e-7;

/// general tolerance of square order
pub const TOLERANCE2: f64 = TOLERANCE * TOLERANCE;

pub use traits::*;

/// B-spline curve
/// # Examples
/// ```
/// use truck_geometry::*;
///
/// // the knot vector
/// let knot_vec = KnotVec::from(
///     vec![0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0]
/// );
///
/// // sign up the control points in the vector of all points
/// let ctrl_pts = vec![ // the vector of the indices of control points
///     Vector4::new(0.0, -2.0, 0.0, 2.0),
///     Vector4::new(1.0, -1.0, 0.0, 1.0),
///     Vector4::new(1.0, 0.0, 0.0, 1.0),
///     Vector4::new(1.0, 1.0, 0.0, 1.0),
///     Vector4::new(0.0, 2.0, 0.0, 2.0),
///     Vector4::new(-1.0, 1.0, 0.0, 1.0),
///     Vector4::new(-1.0, 0.0, 0.0, 1.0),
///     Vector4::new(-1.0, -1.0, 0.0, 1.0),
///     Vector4::new(0.0, -2.0, 0.0, 2.0),
/// ];
///
/// // construct the B-spline curve
/// let bspline = BSplineCurve::new(knot_vec, ctrl_pts);
///
/// // This B-spline curve is a nurbs representation of the unit circle.
/// const N : usize = 100; // sample size in test
/// for i in 0..N {
///     let t = 1.0 / (N as f64) * (i as f64);
///     let v = bspline.subs(t); // We can use the instances as a function.
///     let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2);
///     f64::assert_near2(&c, &1.0);
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BSplineCurve<V> {
    knot_vec: KnotVec,      // the knot vector
    control_points: Vec<V>, // the indices of control points
}

/// Curve for the recursive concatting.
/// # Examples
/// ```
/// use truck_geometry::*;
/// use std::convert::TryInto;
/// let mut cc = CurveCollector::<Vector2>::Singleton;
/// cc.concat(&mut BSplineCurve::new(
///     KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]),
///     vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)],
/// ));
/// cc.concat(&mut BSplineCurve::new(
///     KnotVec::from(vec![1.0, 1.0, 2.0, 2.0]),
///     vec![Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0)],
/// ));
/// let res: BSplineCurve<Vector2> = cc.try_into().unwrap();
/// let line = BSplineCurve::new(
///     KnotVec::from(vec![0.0, 0.0, 2.0, 2.0]),
///     vec![Vector2::new(0.0, 0.0), Vector2::new(2.0, 2.0)],
/// );
/// assert!(res.near2_as_curve(&line));
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum CurveCollector<V> {
    /// the empty curve
    Singleton,
    /// a non-empty curve
    Curve(BSplineCurve<V>),
}

/// B-spline surface
/// # Examples
/// ```
/// use truck_geometry::*;
/// const N : usize = 100; // sample size in test
///
/// // the knot vectors
/// let knot_vec0 = KnotVec::bezier_knot(3);
/// let knot_vec1 = KnotVec::from(
///     vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0]
/// );
/// let knot_vecs = (knot_vec0, knot_vec1);
///
/// // the control points
/// let mut v = vec![vec![Vector4::zero(); 7]; 4];
/// v[0][0] = Vector4::new(0.0, 0.0, 1.0, 1.0);
/// v[0][1] = &v[0][0] / 3.0;
/// v[0][2] = v[0][1].clone();
/// v[0][3] = v[0][0].clone();
/// v[0][4] = v[0][1].clone();
/// v[0][5] = v[0][1].clone();
/// v[0][6] = v[0][0].clone();
/// v[1][0] = Vector4::new(2.0, 0.0, 1.0, 1.0) / 3.0;
/// v[1][1] = Vector4::new(2.0, 4.0, 1.0, 1.0) / 9.0;
/// v[1][2] = Vector4::new(-2.0, 4.0, 1.0, 1.0) / 9.0;
/// v[1][3] = Vector4::new(-2.0, 0.0, 1.0, 1.0) / 3.0;
/// v[1][4] = Vector4::new(-2.0, -4.0, 1.0, 1.0) / 9.0;
/// v[1][5] = Vector4::new(2.0, -4.0, 1.0, 1.0) / 9.0;
/// v[1][6] = Vector4::new(2.0, 0.0, 1.0, 1.0) / 3.0;
/// v[2][0] = Vector4::new(2.0, 0.0, -1.0, 1.0) / 3.0;
/// v[2][1] = Vector4::new(2.0, 4.0, -1.0, 1.0) / 9.0;
/// v[2][2] = Vector4::new(-2.0, 4.0, -1.0, 1.0) / 9.0;
/// v[2][3] = Vector4::new(-2.0, 0.0, -1.0, 1.0) / 3.0;
/// v[2][4] = Vector4::new(-2.0, -4.0, -1.0, 1.0) / 9.0;
/// v[2][5] = Vector4::new(2.0, -4.0, -1.0, 1.0) / 9.0;
/// v[2][6] = Vector4::new(2.0, 0.0, -1.0, 1.0) / 3.0;
/// v[3][0] = Vector4::new(0.0, 0.0, -1.0, 1.0);
/// v[3][1] = &v[3][0] / 3.0;
/// v[3][2] = v[3][1].clone();
/// v[3][3] = v[3][0].clone();
/// v[3][4] = v[3][1].clone();
/// v[3][5] = v[3][1].clone();
/// v[3][6] = v[3][0].clone();
///
/// // cunstruct the B-spline curve
/// let bspline = BSplineSurface::new(knot_vecs, v);
///
/// // This B-spline curve is a nurbs representation of the unit sphere.
/// for i in 0..N {
///     for j in 0..N {
///         let u = 1.0 / (N as f64) * (i as f64);
///         let v = 1.0 / (N as f64) * (j as f64);
///         let v = bspline.subs(u, v); // We can use the instances as a function.
///         let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2) + (v[2] / v[3]).powi(2);
///         f64::assert_near2(&c, &1.0);
///     }
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BSplineSurface<V> {
    knot_vecs: (KnotVec, KnotVec),
    control_points: Vec<Vec<V>>,
}

/// Error handler for [`Error`](./errors/enum.Error.html)
pub type Result<T> = std::result::Result<T, crate::errors::Error>;

#[doc(hidden)]
pub mod bounding_box;
#[doc(hidden)]
pub mod bspcurve;
/// Defines some iterators on control points of B-spline surface.
pub mod bspsurface;
/// Enumerats `Error`.
pub mod errors;
#[doc(hidden)]
pub mod knot_vec;
/// Defines traits: [`Tolerance`], [`Origin`], and [`RationalProjective`].
///
/// [`Toleramce`]: ./traits/trait.Tolerance.html
/// [`Origin`]: ./traits/trait.Origin.html
/// [`RationalProjective`]: ./traits/trait.RationalProjective.html
pub mod traits;
