use geom::{Pt};
use polyshape::{Polyshapable, Polyshape, PolyshapeStyle};
use sfml::graphics::{RenderTarget, RenderWindow, ShapeImpl, Text, Transform, Transformable, Font};
use sfml::system::{Vector2f, Vector2i};
use view::{View};
use sfml::graphics::{Vertex, VertexArray, Drawable, Color, RenderStates, PrimitiveType, LinesStrip};

pub struct Plot {
    //f: fn(f64) -> f64,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
    n: u32,
    vp_left: f64,
    vp_top: f64,
    vp_width: f64,
    vp_height: f64,
    va: VertexArray
}

impl Plot {
    pub fn new(f: &Fn(f64) -> f64, left: f64, right: f64, n: u32) -> Plot {
        let black = Color::black();
        let mut va = VertexArray::new().unwrap();
        va.set_primitive_type(LinesStrip);
        let mut top = -1.0e20;
        let mut bottom = 1.0e20;

        for i in 0..n+1 {
            let x = left + (i as f64) / (n as f64) * (right - left);
            let y = f(x);
            if y > top {
                top = y;
            }
            if y < bottom {
                bottom = y;
            }
            va.append(&Vertex::new_with_pos_color(&Vector2f::new(x as f32, y as f32), &black));
        }
        Plot {
            //f: f,
            left: left,
            right: right,
            top: top,
            bottom: bottom, 
            n: n,
            vp_left: 0.0,
            vp_top: 0.0,
            vp_width: 1.0,
            vp_height: 1.0,
            va: va
        }        
    }

    pub fn set_viewport(&mut self, vp_left: f64, vp_top: f64, vp_width: f64, vp_height: f64) {
        self.vp_left = vp_left;
        self.vp_top = vp_top;
        self.vp_width = vp_width;
        self.vp_height = vp_height;
    }

    pub fn transform(&self) -> Transform {
        let mut tr = Transform::new_identity();
        let p = (self.vp_width / (self.right - self.left)) as f32;
        let q = (-self.vp_height / (self.top - self.bottom)) as f32;
        //println!("p = {}, q = {}", p, q);
        //println!("top = {}, bottom = {}", self.top, self.bottom);
        tr.translate(self.vp_left as f32, self.vp_top as f32);
        tr.scale((self.vp_width / (self.right - self.left)) as f32,
                   (-self.vp_height / (self.top - self.bottom)) as f32);
        tr.translate(-self.left as f32, -self.top as f32);
        tr
    }
}

impl<'s> Drawable for Plot {
    fn draw<RT: RenderTarget>(&self, target: &mut RT, states: &mut RenderStates) {
        target.draw_with_renderstates(&self.va, states);
    }
}
