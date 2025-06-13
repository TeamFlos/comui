use macroquad::math::Vec2;

pub type Transform = nalgebra::Matrix3<f32>;
pub type Point = nalgebra::Point2<f32>;

pub fn quad_contains(quad: [Vec2; 4], pos: Vec2) -> bool {
    let [a, b, c, d] = quad;
    let abp = (b - a).perp_dot(pos - a);
    let bcp = (c - b).perp_dot(pos - b);
    let cdp = (d - c).perp_dot(pos - c);
    let dap = (a - d).perp_dot(pos - d);
    (abp >= 0. && bcp >= 0. && cdp >= 0. && dap >= 0.)
        || (abp <= 0. && bcp <= 0. && cdp <= 0. && dap <= 0.)
}
