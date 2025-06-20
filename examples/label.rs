use comui::{
    component::Component,
    components::label::Label,
    layout::{Layout, LayoutBuilder},
    window::Window,
};
use macroquad::prelude::*;
use nalgebra::Matrix3;

struct Main {
    label1: Label,
    label2: Label
}

impl Default for Main {
    fn default() -> Self {
        Self {
            label1: Label::new("你说的对，但是GitHub是由Tom Preston-Werner、Chris Wanstrath、PJ Hyett和Scott Chacon自主研发的一款基于Git的代码托管平台。它发生在一个被称作互联网的全球信息网络中，在这里，被注册为GitHub用户的人将被授予创建和管理仓库的权限，导引开源软件和协作开发。你将扮演一位名为your_username的开发者，创建或加入项目，和他们一起编写代码，找回bug——同时，逐步发掘GitHub社区的真相。".repeat(4).as_str())
                .with_font_size(20.).with_line_height(25.),
                label2: Label::new("UNDERFLOW✋🤚😅😰")
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
        LayoutBuilder::new()
            .at_rect((10.0, 10.0, 100.0, 400.0), &mut self.label1)
            .at_rect((0.0,0.0,400.0,400.0),&mut self.label2)
            .build()
    }
    fn before_render(&mut self, _tr: &comui::utils::Transform, target: &mut Window) {
        self.label1.area_width = Some(target.pixel_width as f32 - 20.0);
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
