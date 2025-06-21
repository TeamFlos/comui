use comui::{
    component::Component,
    components::label::Label,
    layout::{Layout, LayoutBuilder},
    window::Window,
};
use cosmic_text::Align;
use macroquad::{miniquad::window::dpi_scale, prelude::*};
use nalgebra::Matrix3;

struct Main {
    label_left: Label,
    label_right: Label,
    label_centered: Label,
}

const TEST_TEXT: &str = "你说的对，但是GitHub是由Tom Preston-Werner、Chris Wanstrath、PJ Hyett和Scott Chacon自主研发的一款基于Git的代码托管平台。它发生在一个被称作互联网的全球信息网络中，在这里，被注册为GitHub用户的人将被授予创建和管理仓库的权限，导引开源软件和协作开发。你将扮演一位名为TeamFlos的开发者，创建或加入项目，和他们一起编写代码，找回bug——同时，逐步发掘GitHub社区的真相。";

impl Default for Main {
    fn default() -> Self {
        Self {
            label_left: Label::new(TEST_TEXT)
                .with_texture_align((0.5, 0.0))
                .with_font_size(20.)
                .with_line_height(25.),
            label_centered: Label::new("⬆️上面左对齐\n中间↔️中心对齐\n下面右对齐⬇️")
                .with_texture_align((0.5, 0.0))
                .with_align(Align::Center)
                .with_font_size(60.)
                .with_line_height(65.),
            label_right: Label::new(TEST_TEXT)
                .with_texture_align((0.5, 0.0))
                .with_align(Align::Right)
                .with_font_size(20.)
                .with_line_height(25.),
        }
    }
}

impl Layout for Main {
    fn components(
        &mut self,
    ) -> Vec<(
        comui::utils::Transform,
        &mut dyn comui::component::Component,
    )> {
        const GAP: f32 = 5.0;
        let h1 = self.label_left.computed_height() / dpi_scale();
        let h2 = self.label_centered.computed_height() / dpi_scale();
        let center = screen_width() / 2.0;

        LayoutBuilder::new()
            .at_rect((center, 10.0, 100.0, 400.0), &mut self.label_left)
            .at_rect(
                (center, h1 + 10.0 + GAP, 400.0, 400.0),
                &mut self.label_centered,
            )
            .at_rect(
                (center, h1 + h2 + 10.0 + GAP * 2.0, 100., 100.),
                &mut self.label_right,
            )
            .build()
    }

    fn before_render(&mut self, _tr: &comui::utils::Transform, target: &mut Window) {
        let percent = (get_time() % 10. / 10.) * 0.85 + 0.15;
        let width = Some(target.pixel_width as f32 * percent as f32 - 20.0);
        self.label_left.area_width = width;
        self.label_centered.area_width = width;
        self.label_right.area_width = width;
    }
}

fn config() -> macroquad::prelude::Conf {
    macroquad::prelude::Conf {
        high_dpi: true,
        sample_count: 4,
        window_title: "Label example".to_string(),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut main_view = Main::default();
    use tracing_subscriber::layer::SubscriberExt;

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
    )
    .expect("setup tracy layer");

    let mut window = Window::default();
    loop {
        clear_background(BLUE);
        main_view.render(&Matrix3::identity(), &mut window);
        window.update();
        tracing::event!(
            tracing::Level::INFO,
            message = "finished frame",
            tracy.frame_mark = true
        );
        next_frame().await
    }
}
