use idp_proto::PlumHeadSeal;

#[derive(Debug, Clone)]
pub enum Message {
    BackPressed,
    ForwardPressed(PlumHeadSeal),
}
