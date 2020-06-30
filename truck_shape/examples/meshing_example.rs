use std::f64::consts::PI;
use std::fs::DirBuilder;
use std::iter::FromIterator;
use truck_geometry::*;
use truck_polymesh::PolygonMesh;
use truck_shape::elements::{Integrity, TopoGeomIntegrity};
use truck_shape::mesher::Meshed;
use truck_shape::*;
use truck_topology::*;

#[allow(dead_code)]
fn n_gon_prism(builder: &mut Builder, n: usize) -> Solid {
    let v: Vec<_> = (0..n)
        .map(|i| {
            let t = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            builder.vertex(vector_new!(t.sin(), 0.0, t.cos())).unwrap()
        })
        .collect();
    let wire: Wire = (0..n)
        .map(|i| builder.line(v[i], v[(i + 1) % n]).unwrap())
        .collect();
    let face = builder.plane(wire).unwrap();
    builder.tsweep(face, &vector_new!(0, 2, 0)).unwrap()
}

#[allow(dead_code)]
fn cube(builder: &mut Builder) -> Solid {
    let v: Vertex = builder.vertex(vector_new!(0.0, 0.0, 0.0)).unwrap();
    let edge = builder.tsweep(v, &vector_new!(1.0, 0.0, 0.0)).unwrap();
    let face = builder.tsweep(edge, &vector_new!(0.0, 1.0, 0.0)).unwrap();
    builder.tsweep(face, &vector_new!(0.0, 0.0, 1.0)).unwrap()
}

#[allow(dead_code)]
fn bottle(builder: &mut Builder) -> Solid {
    let (width, thick, height) = (6.0, 4.0, 10.0);
    let v0 = builder
        .vertex(vector_new!(-thick / 4.0, 0.0, -width / 2.0))
        .unwrap();
    let v1 = builder
        .vertex(vector_new!(-thick / 4.0, 0.0, width / 2.0))
        .unwrap();
    let transit = vector_new!(-thick / 2.0, 0.0, 0.0);
    let edge0 = builder.circle_arc(v0, v1, &transit).unwrap();
    let edge1 = builder
        .rotated(
            &edge0,
            &vector_new!(0.0, 0.0, 0.0),
            &vector_new!(0.0, 0.0, 1.0),
            PI,
        )
        .unwrap();
    let wire0 = Wire::from_iter(&[edge0]);
    let wire1 = Wire::from_iter(&[edge1]);
    let face = builder.homotopy(&wire0, &wire1).unwrap();
    builder
        .tsweep(face, &vector_new!(0.0, height, 0.0))
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn tsudsumi(builder: &mut Builder) -> Solid {
    let v0 = builder.vertex(vector_new!(1.0, 2.0, 0.0)).unwrap();
    let v1 = builder.vertex(vector_new!(0.0, 0.0, 1.0)).unwrap();
    let edge = builder.line(v0, v1).unwrap();
    let mut shell = builder
        .rsweep(
            edge,
            &vector_new!(0, 0, 0),
            &vector_new!(0, 1, 0),
            PI * 2.0,
        )
        .unwrap();
    let wire = shell.extract_boundaries();
    for mut wire in wire {
        wire.invert();
        shell.push(builder.plane(wire).unwrap());
    }
    Solid::try_new(vec![shell]).unwrap()
}

#[allow(dead_code)]
fn truck3d(builder: &mut Builder) -> Solid {
    let v: Vec<Vertex> = vec![
        builder.vertex(vector_new!(0, 0, 0)).unwrap(),
        builder.vertex(vector_new!(4, 0, 0)).unwrap(),
        builder.vertex(vector_new!(1, 0, 2)).unwrap(),
        builder.vertex(vector_new!(3, 0, 2)).unwrap(),
    ];
    let edge = vec![
        builder.line(v[1], v[0]).unwrap(),
        builder
            .circle_arc(v[3], v[2], &vector_new!(2, 0, 1))
            .unwrap(),
    ];
    let mut shell = builder.homotopy(&edge[0], &edge[1]).unwrap();
    let face1 = builder
        .rotated(
            &shell[0],
            &vector_new!(2.0, 0.0, 3.5),
            &vector_new!(0.0, 1.0, 0.0),
            std::f64::consts::PI,
        )
        .unwrap();
    let wire0 = shell[0].boundary();
    let wire1 = face1.boundary();
    let face2 = builder.homotopy(&wire0[1].inverse(), &wire1[3]).unwrap()[0].clone();
    let face3 = builder.homotopy(&wire0[3].inverse(), &wire1[1]).unwrap()[0].clone();
    shell.append(&mut vec![face1, face2, face3].into());
    builder
        .tsweep(shell, &vector_new!(0, 3, 0))
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn large_box(builder: &mut Builder) -> Solid {
    const N: usize = 100;

    let v: Vec<_> = (0..N)
        .flat_map(|i| (0..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            builder
                .vertex(vector_new!(i as f64, j as f64, 0.0))
                .unwrap()
        })
        .collect();
    let row_edge: Vec<Vec<_>> = (0..N)
        .map(|i| {
            (1..N)
                .map(|j| builder.line(v[i * N + j - 1], v[i * N + j]).unwrap())
                .collect()
        })
        .collect();
    let col_edge: Vec<Vec<_>> = (1..N)
        .map(|i| {
            (0..N)
                .map(|j| {
                    builder
                        .line(v[(i - 1) * N + j], v[(i % N) * N + j])
                        .unwrap()
                })
                .collect()
        })
        .collect();

    let shell: Shell = (1..N)
        .flat_map(|i| (1..N).map(move |j| (i, j)))
        .map(|(i, j)| {
            let wire = Wire::from_iter(&[
                row_edge[i - 1][j - 1],
                col_edge[i - 1][j],
                row_edge[i][j - 1].inverse(),
                col_edge[i - 1][j - 1].inverse(),
            ]);
            builder.plane(wire).unwrap()
        })
        .collect();
    builder
        .tsweep(shell, &vector_new!(0, 0, 1))
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn torus(builder: &mut Builder) -> Shell {
    let v = vec![
        builder.vertex(vector_new!(0.0, 0.0, 1.0)).unwrap(),
        builder.vertex(vector_new!(0.0, 0.0, 3.0)).unwrap(),
    ];
    let wire = Wire::from_iter(&[
        builder
            .circle_arc(v[0], v[1], &vector_new!(0.0, 1.0, 2.0))
            .unwrap(),
        builder
            .circle_arc(v[1], v[0], &vector_new!(0.0, -1.0, 2.0))
            .unwrap(),
    ]);
    builder
        .rsweep(
            wire,
            &vector_new!(0, 0, 0),
            &vector_new!(0, 1, 0),
            std::f64::consts::PI * 2.0,
        )
        .unwrap()
}

#[allow(dead_code)]
fn half_torus(builder: &mut Builder) -> Solid {
    let v = vec![
        builder.vertex(vector_new!(0.0, 0.0, 1.0)).unwrap(),
        builder.vertex(vector_new!(0.0, 0.0, 3.0)).unwrap(),
    ];
    let wire = Wire::from_iter(&[
        builder
            .circle_arc(v[0], v[1], &vector_new!(0.0, 1.0, 2.0))
            .unwrap(),
        builder
            .circle_arc(v[1], v[0], &vector_new!(0.0, -1.0, 2.0))
            .unwrap(),
    ]);
    let face = builder.plane(wire).unwrap();
    builder
        .rsweep(
            face,
            &vector_new!(0, 0, 0),
            &vector_new!(0, 1, 0),
            std::f64::consts::PI,
        )
        .unwrap()
}

#[allow(dead_code)]
fn truck_torus(builder: &mut Builder) -> Solid {
    let v: Vec<Vertex> = vec![
        builder.vertex(vector_new!(0, 0, 4)).unwrap(),
        builder.vertex(vector_new!(4, 0, 4)).unwrap(),
        builder.vertex(vector_new!(1, 0, 6)).unwrap(),
        builder.vertex(vector_new!(3, 0, 6)).unwrap(),
    ];
    let edge = vec![
        builder.line(v[1], v[0]).unwrap(),
        builder
            .circle_arc(v[3], v[2], &vector_new!(2, 0, 5))
            .unwrap(),
    ];
    let mut shell = builder.homotopy(&edge[0], &edge[1]).unwrap();
    let face1 = builder
        .rotated(
            &shell[0],
            &vector_new!(2.0, 0.0, 7.5),
            &vector_new!(0.0, 1.0, 0.0),
            std::f64::consts::PI,
        )
        .unwrap();
    let wire0 = shell[0].boundary();
    let wire1 = face1.boundary();
    let face2 = builder.homotopy(&wire0[1].inverse(), &wire1[3]).unwrap()[0].clone();
    let face3 = builder.homotopy(&wire0[3].inverse(), &wire1[1]).unwrap()[0].clone();
    shell.append(&mut vec![face1, face2, face3].into());
    builder
        .rsweep(shell, &Vector3::zero(), &vector_new!(1, 0, 0), -PI * 2.0)
        .unwrap()
        .pop()
        .unwrap()
}

#[allow(dead_code)]
fn vase(builder: &mut Builder) -> Shell {
    let v0 = builder.vertex(vector_new!(0, 0, 0)).unwrap();
    let v1 = builder.vertex(vector_new!(1, 0, 0)).unwrap();
    let v2 = builder.vertex(vector_new!(1.5, 3.0, 0.0)).unwrap();
    let origin = &Vector3::zero();
    let axis = &vector_new!(0, 1, 0);
    let edge0 = builder.line(v0, v1).unwrap();
    let inter_points = vec![
        vector_new!(2.0, 0.5, 0.0),
        vector_new!(1.2, 3.5, 0.0),
        vector_new!(1.5, 3.5, 0.0),
    ];
    let edge1 = builder.bezier(v1, v2, inter_points).unwrap();
    let wire = Wire::from_iter(&[edge0, edge1]);
    builder.rsweep(wire, origin, axis, -PI * 2.0).unwrap()
}

#[allow(dead_code)]
fn assert_integrity<T: Integrity>(elem: &T, director: &mut Director, filename: &str) {
    let integrity = director.check_integrity(elem);
    assert_eq!(
        integrity,
        TopoGeomIntegrity::Integrate,
        "Integrate Error: {}",
        filename
    );
}

fn output_mesh<F, T>(director: &mut Director, function: F, filename: &str)
where
    F: FnOnce(&mut Builder) -> T,
    T: Meshed<MeshType = PolygonMesh> + Integrity, {
    let path = "./output/".to_string() + filename;
    let instant = std::time::Instant::now();
    let solid = director.building(function);
    //assert_integrity(&solid, director, filename);
    let mesh = director.get_mesher().meshing(&solid, 0.02);
    let end_time = instant.elapsed();
    println!(
        "{}: {}.{:03} sec",
        filename,
        end_time.as_secs(),
        end_time.subsec_nanos() / 1_000_000,
    );
    let file = std::fs::File::create(path).unwrap();
    truck_io::obj::write(&mesh, file).unwrap();
}

fn main() {
    let mut director = Director::new();
    DirBuilder::new().recursive(true).create("output").unwrap();
    output_mesh(&mut director, cube, "cube.obj");
    output_mesh(&mut director, bottle, "bottle.obj");
    output_mesh(&mut director, tsudsumi, "tsudsumi.obj");
    output_mesh(&mut director, truck3d, "truck3d.obj");
    for n in 3..=8 {
        let filename = format!("{}-gon-prism.obj", n);
        output_mesh(&mut director, |d| n_gon_prism(d, n), &filename);
    }
    output_mesh(&mut director, large_box, "large_plane.obj");
    output_mesh(&mut director, torus, "torus.obj");
    output_mesh(&mut director, half_torus, "half_torus.obj");
    output_mesh(&mut director, truck_torus, "truck_torus.obj");
    output_mesh(&mut director, vase, "vase.obj");
}
