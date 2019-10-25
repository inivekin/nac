use rand::prelude::*;

use crate::nac::Node;

#[derive(Debug,Builder,Default,Clone,Copy)]
#[builder(setter(into))]
#[repr(C)]
pub struct Verlet {
    pub position:  Point2,
    #[builder(default="self.get_default_point(0.0,0.0)")]
    delta_position: Point2,
    #[builder(default="self.get_default_point(0.0,0.0)")]
    theta_position: Vector2,
    #[builder(default="false")]
    pub pinned: bool,
}

impl VerletBuilder {
    fn get_default_point(&self, x: f64, y:f64) -> Point2 {
        Point2 {
            x,
            y
        }
    }
}

impl Verlet {
    pub fn new(x: f64, y: f64) -> Self {
        VerletBuilder::default()
            .position(Point2::new(x,y))
            .build()
            .unwrap()
    }

    fn updated(x: f64, y: f64, dx: f64, dy: f64) -> Self {
        VerletBuilder::default()
            .position(Point2::new(x,y))
            .delta_position(Point2::new(dx,dy))
            .build()
            .unwrap()
    }

    pub fn new_pinned(x: f64, y: f64) -> Self {
        VerletBuilder::default()
            .position(Point2::new(x,y))
            .pinned(true)
            .build()
            .unwrap()
    }
}

#[derive(Clone,PartialEq,Debug,Copy,Default)]
#[repr(C)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    fn new(x: f64, y: f64) -> Self {
        Point2 {
            x,
            y
        }
    }
}

type Vector2 = Point2;

// a cloth type constraint function for two nodes
pub fn internode_constraint(node1: &Node<Verlet>, node2: &Node<Verlet>, spacing: f64) -> (Node<Verlet>,Node<Verlet>) {
        let diff_x = node1.data.position.x - node2.data.position.x;
        let diff_y = node1.data.position.y - node2.data.position.y;
        let dist = (diff_x * diff_x + diff_y * diff_y).sqrt();

        let mut diff = 0.0;
        if dist != 0.0
        {
            diff = (spacing - dist) / dist;
        }

        // TODO tear distance stuff: destroy all refcounts

        let px = diff_x * diff * 0.5;
        let py = diff_y * diff * 0.5;


        let node1_constrained: Node<Verlet>;
        if node1.data.pinned {
            node1_constrained = node1.clone();
        }
        else
        {
            node1_constrained = Node::new(
                Verlet::updated(
                    node1.data.position.x + px,
                    node1.data.position.y + py,
                    node1.data.position.x,
                    node1.data.position.y
                )
            );
        }

        let node2_constrained: Node<Verlet>;
        if node2.data.pinned {
            node2_constrained = node2.clone();
        }
        else
        {
            node2_constrained = Node::new(
                Verlet::updated(
                    node2.data.position.x - px,
                    node2.data.position.y - py,
                    node2.data.position.x,
                    node2.data.position.y
                )
            );
        }

        (node1_constrained, node2_constrained)
}

pub fn gravity_constraint(node: &Node<Verlet>, delta: f64, gravity: i16) -> Node<Verlet> {
    let delta = delta.powi(2);
    Node::new(
        Verlet::updated(
            node.data.position.x,
            node.data.position.y + f64::from(gravity) * delta,
            node.data.position.x,
            node.data.position.y
        )
    )
}

pub fn wind_constraint(node: &Node<Verlet>, horz_strength: f64, vert_strength: f64) -> Node<Verlet> {
    let rand_x = horz_strength * rand::thread_rng().gen::<f64>();
    let new_x = if rand_x < -2.5 {
        rand_x
    } else {
        0.0
    };
    let new_x = horz_strength * rand::thread_rng().gen::<f64>();
    let new_y = vert_strength * rand::thread_rng().gen::<f64>();
    Node::new(
        Verlet::updated(
            node.data.position.x + new_x,
            node.data.position.y + new_y,
            node.data.position.x,
            node.data.position.y
        )
    )
}
