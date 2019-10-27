use crate::nac::{Node, SharedNode, SharedNodes, InterConnector, SelfConnector, Mesh};
use crate::verlet::Verlet;
use crate::verlet::{gravity_constraint, wind_constraint, internode_constraint, force_constraint};

use std::sync::{Arc, RwLock};
use std::convert::TryInto;

pub trait Cloth {
    fn new_cloth(height: u8, width: u8, spacing: u8) -> Self;
    fn cloth_boundaries(height: u8, width: u8, spacing: u8) -> Self;
    fn cloth_interweave(height: u8, width: u8, spacing: u8) -> Self;
}

impl Cloth for Mesh<Verlet> {
    // TODO(kevinc) make delarative and not imperative
    fn new_cloth(height: u8, width: u8, spacing: u8) -> Mesh<Verlet> {
        let mut nodes: Vec<SharedNode<Verlet>> = vec!();
        let mut interconnectors: Vec<InterConnector<Verlet>> = vec!();
        let mut selfconnectors: Vec<SelfConnector<Verlet>> = vec!();
        let gravity = move |node: &Node<Verlet>| gravity_constraint(node, 0.0016,1200);
        let wind = move |node: &Node<Verlet>| wind_constraint(node, -3.0,-0.00);
        let cloth_constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,spacing.clone() as f64, 0.5);
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

    fn cloth_boundaries(height: u8, width: u8, spacing: u8) -> Mesh<Verlet> {
        let mut nodes: Vec<SharedNode<Verlet>> = vec!();
        let mut interconnectors: Vec<InterConnector<Verlet>> = vec!();
        let mut selfconnectors: Vec<SelfConnector<Verlet>> = vec!();
        let gravity = move |node: &Node<Verlet>| gravity_constraint(node, 0.0016,1200);
        let wind = move |node: &Node<Verlet>| wind_constraint(node, -3.0,-0.00);
        let cloth_constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,spacing.clone() as f64, 0.5);

        let top_left = Arc::new(RwLock::new(Node::new(Verlet::new_pinned(f64::from(0),f64::from(0)))));
        let top_right = Arc::new(RwLock::new(Node::new(Verlet::new_pinned(f64::from(width*spacing),f64::from(0)))));
        let bottom_left = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(0),f64::from(spacing*height)))));
        let bottom_right = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(spacing*width),f64::from(spacing*height)))));

        for x in 1..width {
            let p_up = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(x*spacing),f64::from(0)))));
            if x == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p_up), Some(vec!(Arc::clone(&top_left))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p_up), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p_up),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            
            nodes.push(p_up);
        }
        let interconnector = InterConnector::new(Arc::clone(&top_right), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);

        let selfconnector = SelfConnector::new(Arc::clone(&top_right),Arc::new(gravity));
        selfconnectors.push(selfconnector);
        
        for x in 1..width {
            let p = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(x*spacing),f64::from(height*spacing)))));
            if x == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(&bottom_left))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            let selfconnector = SelfConnector::new(Arc::clone(&p),Arc::new(wind));
            selfconnectors.push(selfconnector);
            
            nodes.push(p);
        }
        let interconnector = InterConnector::new(Arc::clone(&bottom_right), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);

        let selfconnector = SelfConnector::new(Arc::clone(&bottom_right),Arc::new(gravity));
        selfconnectors.push(selfconnector);
        let selfconnector = SelfConnector::new(Arc::clone(&bottom_right),Arc::new(wind));
        selfconnectors.push(selfconnector);
        


        for y in 1..height {
            let p = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(0),f64::from(y*spacing)))));
            if y == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(&top_left))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            
            nodes.push(p);
        }
        let interconnector = InterConnector::new(Arc::clone(&bottom_left), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);

        let selfconnector = SelfConnector::new(Arc::clone(&bottom_left),Arc::new(gravity));
        selfconnectors.push(selfconnector);


        for y in 1..height {
            let p = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(width*spacing),f64::from(y*spacing)))));
            if y == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(&top_right))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            
            nodes.push(p);
        }
        let interconnector = InterConnector::new(Arc::clone(&bottom_right), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);

        let selfconnector = SelfConnector::new(Arc::clone(&bottom_right),Arc::new(gravity));
        selfconnectors.push(selfconnector);

        nodes.push(top_left);
        nodes.push(bottom_right);
        nodes.push(top_right); 
        nodes.push(bottom_left);

        Mesh::new(nodes, interconnectors, selfconnectors)
    }

    fn cloth_interweave(height: u8, width: u8, spacing: u8) -> Mesh<Verlet> {
        let mut nodes: Vec<SharedNode<Verlet>> = vec!();
        let mut interconnectors: Vec<InterConnector<Verlet>> = vec!();
        let mut selfconnectors: Vec<SelfConnector<Verlet>> = vec!();
        let gravity = move |node: &Node<Verlet>| gravity_constraint(node, 0.00016,1200);
        let wind = move |node: &Node<Verlet>| wind_constraint(node, -4.0,-0.00);
        let cloth_constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,spacing.clone() as f64, 0.5);

        let top_left = Arc::new(RwLock::new(Node::new(Verlet::new_pinned(f64::from(0),f64::from(0)))));
        let top_right = Arc::new(RwLock::new(Node::new(Verlet::new_pinned(f64::from(width*spacing),f64::from(0)))));
        let bottom_left = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(0),f64::from(spacing*height)))));
        let bottom_right = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(spacing*width),f64::from(spacing*height)))));

        for x in 1..width {
            let p_up = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(x*spacing),f64::from(0)))));
            if x == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p_up), Some(vec!(Arc::clone(&top_left))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p_up), Some(vec!(Arc::clone(&nodes[(nodes.len() - 2) as usize]))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p_up),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            
            let p_down = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(x*spacing),f64::from(height*spacing)))));
            if x == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p_down), Some(vec!(Arc::clone(&bottom_left))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p_down), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p_down),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            let selfconnector = SelfConnector::new(Arc::clone(&p_down),Arc::new(wind));
            selfconnectors.push(selfconnector);

            let height_constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,(spacing*height) as f64, 0.5);
            let interconnector = InterConnector::new(Arc::clone(&p_down), Some(vec!(Arc::clone(&p_up))),Arc::new(height_constraint));
            interconnectors.push(interconnector);
            
            nodes.push(p_up);
            nodes.push(p_down);
        }
        let interconnector = InterConnector::new(Arc::clone(&bottom_right), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);

        let interconnector = InterConnector::new(Arc::clone(&top_right), Some(vec!(Arc::clone(&nodes[(nodes.len() - 2) as usize]))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);



        let selfconnector = SelfConnector::new(Arc::clone(&bottom_right),Arc::new(gravity));
        selfconnectors.push(selfconnector);
        let selfconnector = SelfConnector::new(Arc::clone(&bottom_right),Arc::new(wind));
        selfconnectors.push(selfconnector);
        


        for y in 1..height {
            let p_left = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(0),f64::from(y*spacing)))));
            if y == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p_left), Some(vec!(Arc::clone(&top_left))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p_left), Some(vec!(Arc::clone(&nodes[(nodes.len() - 2) as usize]))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p_left),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            
            let p_right = Arc::new(RwLock::new(Node::new(Verlet::new(f64::from(width*spacing),f64::from(y*spacing)))));
            if y == 1 {
                let interconnector = InterConnector::new(Arc::clone(&p_right), Some(vec!(Arc::clone(&top_right))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            } else {
                let interconnector = InterConnector::new(Arc::clone(&p_right), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
                interconnectors.push(interconnector);
            }

            let selfconnector = SelfConnector::new(Arc::clone(&p_right),Arc::new(gravity));
            selfconnectors.push(selfconnector);
            
            let width_constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,(spacing*width) as f64, 0.5);
            let interconnector = InterConnector::new(Arc::clone(&p_left), Some(vec!(Arc::clone(&p_right))),Arc::new(width_constraint));
            interconnectors.push(interconnector);

            nodes.push(p_left);
            nodes.push(p_right);
        }
        let interconnector = InterConnector::new(Arc::clone(&bottom_right), Some(vec!(Arc::clone(nodes.last().unwrap()))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);
        let interconnector = InterConnector::new(Arc::clone(&bottom_left), Some(vec!(Arc::clone(&nodes[(nodes.len() - 2) as usize]))),Arc::new(cloth_constraint));
        interconnectors.push(interconnector);

        let selfconnector = SelfConnector::new(Arc::clone(&bottom_left),Arc::new(gravity));
        selfconnectors.push(selfconnector);


        let selfconnector = SelfConnector::new(Arc::clone(&bottom_right),Arc::new(gravity));
        selfconnectors.push(selfconnector);

        nodes.push(top_left);
        nodes.push(bottom_right);
        nodes.push(top_right); 
        nodes.push(bottom_left);

        Mesh::new(nodes, interconnectors, selfconnectors)
    }
}


#[no_mangle]
pub unsafe extern fn create_verlet_node(x: f64, y: f64) -> *mut SharedNode<Verlet> {
    Box::into_raw(Box::new(Arc::new(RwLock::new(Node::new(Verlet::new(x,y))))))
}

#[no_mangle]
pub unsafe extern fn add_impetus(mesh_ptr: *mut Mesh<Verlet>, node_ptr: *mut SharedNode<Verlet>, delta: f64, x_force: f64, y_force: f64) {
    if !mesh_ptr.is_null() & !node_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        let node = node_ptr as *mut SharedNode<Verlet>;

        let constraint = move |node: &Node<Verlet>| force_constraint(node, delta, x_force, y_force);

        (*mesh).selfconnectors.push(SelfConnector::new(Arc::clone(&(*node)), Arc::new(constraint)));
    }
}

/*#[no_mangle]
pub unsafe extern init_node_group() -> *mut Option<SharedNodes<Verlet>> {
    None
}

#[no_mangle]
pub unsafe extern add_to_node_group(node_group: *mut Option<SharedNodes<Verlet>>, node: *mut SharedNode<Verlet>) {
    if !node_group.is_null() & !node.is_null() {
        let node_group = node_group as *mut Option<SharedNodes<Verlet>>;
        let node = node as *mut SharedNode<Verlet>;

        if let Some(nodes) = node_group {
            nodes.push(Arc::clone(&(*node)));
        } else {
            nodes = vec!(Arc::clone(&(*node)));
        }
    } 
}

#[no_mangle]
pub unsafe extern fn add_multiconnector(mesh_ptr: *mut Mesh<Verlet>, primary_node_ptr: *mut SharedNode<Verlet>, node_group_ptr: *mut Option<SharedNodes<Verlet>>, distance: f64, spring: f64) {
    if !mesh_ptr.is_null() & !primary_node_ptr.is_null() & !node_group_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        let primary_node = primary_node_ptr as *mut SharedNode<Verlet>;
        let node_group = node_group_ptr as *mut Option<SharedNodes<Verlet>>;

        let constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,distance,spring);

        (*mesh).interconnectors.push(InterConnector::new(Arc::clone(&(*primary_node)), (*node_group), Arc::new(constraint)));
    }               
}*/

#[no_mangle]
pub unsafe extern fn add_connector(mesh_ptr: *mut Mesh<Verlet>, primary_node_ptr: *mut SharedNode<Verlet>, secondary_node_ptr: *mut SharedNode<Verlet>, distance: f64, spring: f64) {
    if !mesh_ptr.is_null() & !primary_node_ptr.is_null() & !secondary_node_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        let primary_node = primary_node_ptr as *mut SharedNode<Verlet>;
        let secondary_node = secondary_node_ptr as *mut SharedNode<Verlet>;

        let constraint = move |node1: &Node<Verlet>, node2: &Node<Verlet>| internode_constraint(node1,node2,distance,spring);

        (*mesh).interconnectors.push(InterConnector::new(Arc::clone(&(*primary_node)), Some(vec!(Arc::clone(&(*secondary_node)))), Arc::new(constraint)));
    }               
}

#[no_mangle]
pub unsafe extern fn get_cloth_mesh(h: u8, w: u8, s: u8)
    -> *mut Mesh<Verlet>
{
    let mesh = Cloth::cloth_boundaries(w, h, s);
    Box::into_raw(Box::new(mesh)) as *mut Mesh<Verlet>
}

#[no_mangle]
pub unsafe extern fn get_woven_cloth_mesh(h: u8, w: u8, s: u8)
    -> *mut Mesh<Verlet>
{
    let mesh = Cloth::cloth_interweave(w, h, s);
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
#[repr(C)]
pub struct DualCPoint2 {
    x: f64,
    dx: f64,
    y: f64,
    dy: f64
}

// NOTE vector pointer not being reliably unpacked by python
#[no_mangle]
pub unsafe extern fn get_cloth_mesh_positions(mesh_ptr: *mut Mesh<Verlet>)
    -> CVecView {
    if !mesh_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        let positions: Vec<CPoint2> = (*mesh).nodes.iter().map( |node|
            if let Ok(borrowed_node) = node.read() {
                println!("{:?}, {:?}",borrowed_node.data.position.x,borrowed_node.data.position.y);
                CPoint2 {
                    x: borrowed_node.data.position.x.clone(),
                    y: borrowed_node.data.position.y.clone()
                }
            } else { // FIXME(kevinc) return railed result instead of incorrect values
                CPoint2 {
                    x: 9999.0,
                    y: 9999.0
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
pub unsafe extern fn get_cloth_interconnector_count(mesh_ptr: *mut Mesh<Verlet>)
    -> usize {
    //if !mesh_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        (*mesh).interconnectors.len()
    //}
}
#[no_mangle]
pub unsafe extern fn get_interconnector_constraint_count(connector_ptr: *mut Mesh<Verlet>, index: usize) -> usize {
    if !connector_ptr.is_null() {
        let connector = connector_ptr as *mut Mesh<Verlet>;
        if let Some(constraints) = &(*connector).interconnectors[index].constraints {
            constraints.len()
        } else {
            0
        }
    } else {
        0
    }
}
#[no_mangle]
pub unsafe extern fn get_interconnector_constraint(connector_ptr: *mut Mesh<Verlet>, inter_index: usize, index: usize) -> DualCPoint2 {
    if !connector_ptr.is_null() {
        let connector = connector_ptr as *mut Mesh<Verlet>;
        if let Some(constraints) = &(*connector).interconnectors[inter_index].constraints {
            (DualCPoint2 {
                dx: constraints[index].read().unwrap().data.position.x,
                dy: constraints[index].read().unwrap().data.position.y,
                x: (*connector).interconnectors[inter_index].node.read().unwrap().data.position.x,
                y: (*connector).interconnectors[inter_index].node.read().unwrap().data.position.y
            })
        } else {
            (DualCPoint2 {
                dx: 0.0,
                dy: 0.0,
                x: (*connector).interconnectors[inter_index].node.read().unwrap().data.position.x,
                y: (*connector).interconnectors[inter_index].node.read().unwrap().data.position.y
            })
        } 
    } else {
        (DualCPoint2 {
            dx: 0.0,
            dy: 0.0,
            x: 0.0,
            y: 0.0
        })
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

#[no_mangle]
pub unsafe extern fn mesh_node_position(mesh_ptr: *mut Mesh<Verlet>, idx: usize)
    -> CPoint2 {
    if !mesh_ptr.is_null() {
        let mesh = mesh_ptr as *mut Mesh<Verlet>;
        if let Ok(borrowed_node) = (*mesh).nodes[idx].read() {
            CPoint2 {
                x: borrowed_node.data.position.x.clone(),
                y: borrowed_node.data.position.y.clone()
            }
        } else { // FIXME(kevinc) return railed result instead of incorrect values
            CPoint2 {
                x: 9999.0,
                y: 9999.0
            }
        }
    } else {
        CPoint2 {
            x: 9999.0,
            y: 9999.0
        }
    }
}
