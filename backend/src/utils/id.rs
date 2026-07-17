pub struct DeviceId(Uuid);
pub struct SessionId(Uuid);
pub struct TransferId(Uuid);
pub struct PeerId(Uuid);

impl DeviceId {
    pub fn new() -> Self;
}
impl SessionId {
    pub fn new() -> Self;
}
impl TransferId {
    pub fn new() -> Self;
}
impl PeerId {
    pub fn new() -> Self;
}