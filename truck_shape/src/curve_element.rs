use crate::errors::Error;
use crate::{Director, Result};
use geometry::{BSplineCurve, BSplineSurface};
use topology::*;

pub trait CurveElement: Sized {
    fn front_vertex(&self) -> Vertex;
    fn back_vertex(&self) -> Vertex;
    fn get_geometry(&self, director: &mut Director) -> Result<BSplineCurve>;
    fn clone_wire(&self) -> Wire;
    fn for_each<F: FnMut(&Edge)>(&self, closure: F);
    fn is_closed(&self) -> bool;
    fn split_wire(&self) -> Option<[Wire; 2]>;
    fn homotopy<T>(&self, other: &T, director: &mut Director) -> Result<Shell>
    where T: CurveElement {
        let closed0 = self.is_closed();
        let closed1 = other.is_closed();
        if closed0 && closed1 {
            closed_homotopy(self, other, director)
        } else if !closed0 && !closed1 {
            open_homotopy(self, other, director)
        } else {
            Err(Error::DifferentHomotopyType)
        }
    }
}

fn open_homotopy<C0, C1>(elem0: &C0, elem1: &C1, director: &mut Director) -> Result<Shell>
where
    C0: CurveElement,
    C1: CurveElement, {
    let curve0 = elem0.get_geometry(director)?;
    let curve1 = elem1.get_geometry(director)?;
    let surface = BSplineSurface::homotopy(curve0, curve1);
    let edge0 = director
        .get_builder()
        .line(elem0.back_vertex(), elem1.back_vertex())?;
    let edge1 = director
        .get_builder()
        .line(elem1.front_vertex(), elem0.front_vertex())?;
    let mut wire = elem0.clone_wire();
    wire.push_back(edge0);
    elem1.for_each(|edge| wire.push_back(edge.inverse()));
    wire.push_back(edge1);
    let face = Face::try_new(wire)?;
    director.insert(&face, surface);
    Ok(vec![face].into())
}

fn closed_homotopy<C0, C1>(elem0: &C0, elem1: &C1, director: &mut Director) -> Result<Shell>
where
    C0: CurveElement,
    C1: CurveElement, {
    let [mut wire0, mut wire1] = elem0.split_wire().unwrap();
    let [mut wire2, mut wire3] = elem1.split_wire().unwrap();
    let curve0 = wire0.get_geometry(director)?.clone();
    let curve2 = wire2.get_geometry(director)?.clone();
    let surface0 = BSplineSurface::homotopy(curve0, curve2);
    let curve1 = wire1.get_geometry(director)?.clone();
    let curve3 = wire3.get_geometry(director)?.clone();
    let surface1 = BSplineSurface::homotopy(curve1, curve3);
    let edge0 = director
        .get_builder()
        .line(wire0.front_vertex().unwrap(), wire2.front_vertex().unwrap())?;
    let edge1 = director
        .get_builder()
        .line(wire0.back_vertex().unwrap(), wire2.back_vertex().unwrap())?;
    wire0.push_back(edge1);
    wire0.append(wire2.inverse());
    wire0.push_back(edge0.inverse());
    wire1.push_back(edge0);
    wire1.append(wire3.inverse());
    wire1.push_back(edge1.inverse());
    let face0 = Face::try_new(wire0)?;
    let face1 = Face::try_new(wire1)?;
    director.insert(&face0, surface0);
    director.insert(&face1, surface1);
    Ok(vec![face0, face1].into())
}

impl CurveElement for Edge {
    fn front_vertex(&self) -> Vertex { self.front() }
    fn back_vertex(&self) -> Vertex { self.back() }
    fn get_geometry(&self, director: &mut Director) -> Result<BSplineCurve> {
        director.get_oriented_curve(self)
    }
    fn clone_wire(&self) -> Wire { Wire::by_slice(&[*self]) }
    fn for_each<F: FnMut(&Edge)>(&self, mut closure: F) { closure(self) }
    fn is_closed(&self) -> bool { false }
    fn split_wire(&self) -> Option<[Wire; 2]> { None }
}

impl CurveElement for Wire {
    fn front_vertex(&self) -> Vertex { self.front_vertex().unwrap() }
    fn back_vertex(&self) -> Vertex { self.back_vertex().unwrap() }
    fn get_geometry(&self, director: &mut Director) -> Result<BSplineCurve> {
        director.bspline_by_wire(self)
    }
    fn clone_wire(&self) -> Wire { self.clone() }
    fn for_each<F: FnMut(&Edge)>(&self, closure: F) { self.edge_iter().for_each(closure) }
    fn is_closed(&self) -> bool { self.is_closed() }
    fn split_wire(&self) -> Option<[Wire; 2]> {
        if self.len() < 2 {
            None
        } else {
            let mut part0 = self.clone();
            let part1 = part0.split_off(self.len() / 2);
            Some([part0, part1])
        }
    }
}
