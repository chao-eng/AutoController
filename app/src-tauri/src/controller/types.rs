use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControllerType {
    Xbox360,
    DualShock4,
}

impl std::fmt::Display for ControllerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControllerType::Xbox360 => write!(f, "Xbox 360"),
            ControllerType::DualShock4 => write!(f, "DualShock 4"),
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Buttons: u16 {
        const DPAD_UP = 0x0001;
        const DPAD_DOWN = 0x0002;
        const DPAD_LEFT = 0x0004;
        const DPAD_RIGHT = 0x0008;
        const START = 0x0010;
        const BACK = 0x0020;
        const LEFT_THUMB = 0x0040;
        const RIGHT_THUMB = 0x0080;
        const LEFT_SHOULDER = 0x0100;
        const RIGHT_SHOULDER = 0x0200;
        const GUIDE = 0x0400;
        const A = 0x1000;
        const B = 0x2000;
        const X = 0x4000;
        const Y = 0x8000;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Button {
    A,
    B,
    X,
    Y,
    LB,
    RB,
    LT,
    RT,
    Back,
    Start,
    Guide,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

impl Button {
    pub fn to_flag(&self) -> u16 {
        match self {
            Button::A => Buttons::A.bits(),
            Button::B => Buttons::B.bits(),
            Button::X => Buttons::X.bits(),
            Button::Y => Buttons::Y.bits(),
            Button::LB => Buttons::LEFT_SHOULDER.bits(),
            Button::RB => Buttons::RIGHT_SHOULDER.bits(),
            Button::Back => Buttons::BACK.bits(),
            Button::Start => Buttons::START.bits(),
            Button::Guide => Buttons::GUIDE.bits(),
            Button::LeftThumb => Buttons::LEFT_THUMB.bits(),
            Button::RightThumb => Buttons::RIGHT_THUMB.bits(),
            Button::DPadUp => Buttons::DPAD_UP.bits(),
            Button::DPadDown => Buttons::DPAD_DOWN.bits(),
            Button::DPadLeft => Buttons::DPAD_LEFT.bits(),
            Button::DPadRight => Buttons::DPAD_RIGHT.bits(),
            Button::LT | Button::RT => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ThumbAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TriggerSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerState {
    pub buttons: u16,
    pub left_thumb_x: i16,
    pub left_thumb_y: i16,
    pub right_thumb_x: i16,
    pub right_thumb_y: i16,
    pub left_trigger: u8,
    pub right_trigger: u8,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            buttons: 0,
            left_thumb_x: 0,
            left_thumb_y: 0,
            right_thumb_x: 0,
            right_thumb_y: 0,
            left_trigger: 0,
            right_trigger: 0,
        }
    }
}

impl From<&ControllerState> for crate::controller::vigem::XusbReport {
    fn from(state: &ControllerState) -> Self {
        Self {
            w_buttons: state.buttons,
            b_left_trigger: state.left_trigger,
            b_right_trigger: state.right_trigger,
            s_thumb_lx: state.left_thumb_x,
            s_thumb_ly: state.left_thumb_y,
            s_thumb_rx: state.right_thumb_x,
            s_thumb_ry: state.right_thumb_y,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub index: usize,
    pub controller_type: ControllerType,
    pub connected: bool,
    pub state: ControllerState,
    pub vigem_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub id: String,
    pub index: usize,
    pub controller_type: ControllerType,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViGEmStatus {
    pub available: bool,
    pub driver_installed: bool,
    pub dll_found: bool,
    pub connected: bool,
    pub error_code: Option<u32>,
    pub message: String,
}
