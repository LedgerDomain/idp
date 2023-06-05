use iced::Element;
use idp_proto::PlumHeadSeal;

use crate::Message;

pub struct PlumHeadSealView;

impl PlumHeadSealView {
    pub fn view(&self, plum_head_seal: &PlumHeadSeal, _debug: bool) -> Element<Message> {
        let mut plum_head_seal_string = plum_head_seal.to_string();
        plum_head_seal_string.truncate(8);
        iced::widget::button::Button::new(iced::widget::text(plum_head_seal_string))
            .on_press(Message::ForwardPressed(plum_head_seal.clone()))
            .into()
    }
}
