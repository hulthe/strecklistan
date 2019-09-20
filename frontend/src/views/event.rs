use crate::app::Msg;
use crate::generated::css_classes::C;
use crate::models::event::Event;
use seed::prelude::*;
use seed::*;

impl Event {
    pub fn view(&self, fade: bool) -> Node<Msg> {
        div![
            class![
                C.p_6,
                C.max_w_md,
                C.text_center,
                C.bg_teal_light,
                C.rounded,
                C.shadow_md,
                C.m_8,
                if fade { C.opacity_75 } else { C.opacity_100 },
            ],
            div![class![C.text_3xl, C.font_bold,], self.title,],
            div![
                //TODO: self.description
                "Dolore ad magnam quia aliquam hic et quam. Et nisi dignissimos veniam sit eos quam\
                enim aut. Unde sunt tenetur sunt consequatur. Eum illo voluptatum incidunt\
                molestias ad voluptatem rerum. Et praesentium tempora qui omnis iusto voluptas in.\
                Vel velit ea consequatur velit vero dolorum."
            ],
            div![
                class![C.flex, C.text_center, C.text_2xl,],
                div![class![C.flex_1,], format!("{}", self.start_time),],
                div![class![C.flex_1,], format!("{}", self.location),],
            ],
        ]
    }
}
