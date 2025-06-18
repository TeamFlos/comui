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

pub fn cosmic_color_to_macroquad_color(color: cosmic_text::Color) -> macroquad::color::Color {
    macroquad::color::Color {
        r: (color.r() as f32 / 255.0),
        g: (color.g() as f32 / 255.0),
        b: (color.b() as f32 / 255.0),
        a: (color.a() as f32 / 255.0),
    }
}

pub fn macroquad_color_to_cosmic_color(color: macroquad::color::Color) -> cosmic_text::Color {
    cosmic_text::Color::rgba(
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
        (color.r * 255.0) as u8,
        (color.a * 255.0) as u8,
    )
}
