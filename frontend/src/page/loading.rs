use crate::generated::css_classes::C;
use seed::prelude::*;
use seed::*;

pub struct Loading;

impl Loading {
    pub fn view<M>() -> Node<M> {
        div![C![C.penguin, C.margin_hcenter]]
    }
}
