use crate::nac::{Node, SharedNode, InterConnector, SelfConnector, Mesh};
use crate::verlet::Verlet;
use crate::verlet::{gravity_constraint, wind_constraint, internode_constraint};

use std::sync::{Arc, RwLock};
use std::convert::TryInto;

pub trait Cloth {
    fn new_cloth(height: u8, width: u8, spacing: u8) -> Self;
}

impl Cloth for Mesh<Verlet> {
    // TODO(kevinc) make delarative and not imperative
    fn new_cloth(height: u8, width: u8, spacing: u8) -> Mesh<Verlet> {
        let mut nodes: Vec<SharedNode<Verlet>> = vec!();
        let mut interconnectors: Vec<InterConnector<Verlet>> = vec!();
        let mut selfconnectors: Vec<SelfConnector<Verlet>> = vec!();
        let gravity = move |node: &Node<Verlet>| gravity_constraint(node, 0.0016,12000);
        let wind = move |node: &Node<Verlet>| wind_constraint(node, -3.0,-0.01);
        let cloth_constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,spacing.clone() as f64);
        for y in 0..height {
            for x in 0..width {
                let p = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(x*spacing),f64::from(y*spacing)))));

                let selfconnector = SelfConnector::new(Arc::clone(&p),Arc::new(gravity));
                selfconnectors.push(selfconnector);


                if x != 0 {
                   let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
                    interconnectors.push(interconnector);
                }
                if y == 0 {
                    let p = Arc::new(RwLock::new(Node::new(Verlet::new_pinned(f64::from(spacing*x),f64::from(0.0)))));
                    nodes.push(p);
                } else {
                    nodes.push(Arc::clone(&p));
                }
                if y != 0 {
                   let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(&nodes[(x+(y-1) * width) as usize]))), Arc::new(cloth_constraint));
                    interconnectors.push(interconnector);
                }
                if (x != 0) & (y != 0) {
                   let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(nodes.last().unwrap()),Arc::clone(&nodes[(x+(y-1) * width) as usize]))), Arc::new(cloth_constraint));
                    interconnectors.push(interconnector);
                }

                let selfconnector = SelfConnector::new(Arc::clone(&p),Arc::new(wind));
                selfconnectors.push(selfconnector);
            }
        }

        Mesh::new(nodes, interconnectors, selfconnectors)

    }
}

#[no_mangle]
pub unsafe extern fn get_cloth_mesh(h: u8, w: u8, s: u8)
    -> *mut Mesh<Verlet>
{
    let mesh = Cloth::new_cloth(w, h, s);
    Box::into_raw(Box::new(mesh)) as *mut Mesh<Verlet>
}

#[no_mangle]
pub unsafe extern fn update_cloth_mesh(mesh_ptr: *mut Mesh<Verlet>, delta: f64, physics_accuracy: u8) {
    if !mesh_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        (*mesh).update(delta, physics_accuracy)
    }
}

#[repr(C)]
pub struct CVecView {
    array: *mut CPoint2,
    size: usize,
}

#[repr(C)]
pub struct CPoint2 {
    x: f64,
    y: f64
}

#[no_mangle]
pub unsafe extern fn get_cloth_mesh_positions(mesh_ptr: *mut Mesh<Verlet>)
    -> CVecView {
    if !mesh_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        let positions: Vec<CPoint2> = (*mesh).nodes.iter().map( |node|
            if let Ok(borrowed_node) = node.read() {
                CPoint2 {
                    x: borrowed_node.data.position.x.clone(),
                    y: borrowed_node.data.position.y.clone()
                }
            } else { // FIXME(kevinc) return railed result instead of incorrect values
                CPoint2 {
                    x: 0.0,
                    y: 0.0
                }
            }).collect();
        let positions_length: usize = positions.len();
        CVecView {
            array: Box::into_raw(Box::new(positions)) as *mut CPoint2,
            size: positions_length
            }
    } else {
        let positions: Vec<CPoint2> = vec!();
        let positions_length = 0;
        CVecView {
            array: Box::into_raw(Box::new(positions)) as *mut CPoint2,
            size: positions_length
        }
    }
}

#[no_mangle]
pub unsafe extern fn vector_free(vecview: CVecView) {
    Box::from_raw(vecview.array);
}

#[no_mangle]
pub unsafe extern fn mesh_free(ssv: *mut Mesh<Verlet>) {
    if !ssv.is_null() {
        let sv = ssv as *mut Mesh<Verlet>;
        Box::from_raw(sv);
    }
}

#[no_mangle]
pub unsafe extern fn mesh_node_count(ssv: *const Mesh<Verlet>)
    -> u32
{
    let sv = ssv as *mut Mesh<Verlet>;
    (*sv).nodes.len().try_into().unwrap()
}
