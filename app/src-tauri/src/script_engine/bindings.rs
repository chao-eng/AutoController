use crate::controller::{Button, ThumbAxis, TriggerSide};

pub(super) fn parse_button(btn: &str) -> Option<Button> {
    match btn.to_uppercase().as_str() {
        "A" => Some(Button::A),
        "B" => Some(Button::B),
        "X" => Some(Button::X),
        "Y" => Some(Button::Y),
        "LB" => Some(Button::LB),
        "RB" => Some(Button::RB),
        "LT" => Some(Button::LT),
        "RT" => Some(Button::RT),
        "BACK" => Some(Button::Back),
        "START" => Some(Button::Start),
        "GUIDE" => Some(Button::Guide),
        "LS" | "L3" => Some(Button::LeftThumb),
        "RS" | "R3" => Some(Button::RightThumb),
        "UP" | "DPAD_UP" => Some(Button::DPadUp),
        "DOWN" | "DPAD_DOWN" => Some(Button::DPadDown),
        "LEFT" | "DPAD_LEFT" => Some(Button::DPadLeft),
        "RIGHT" | "DPAD_RIGHT" => Some(Button::DPadRight),
        _ => None,
    }
}

pub(super) fn parse_axis(axis: &str) -> Option<ThumbAxis> {
    match axis.to_lowercase().as_str() {
        "leftx" | "lx" => Some(ThumbAxis::LeftX),
        "lefty" | "ly" => Some(ThumbAxis::LeftY),
        "rightx" | "rx" => Some(ThumbAxis::RightX),
        "righty" | "ry" => Some(ThumbAxis::RightY),
        _ => None,
    }
}

pub(super) fn parse_trigger(side: &str) -> Option<TriggerSide> {
    match side.to_lowercase().as_str() {
        "left" | "l" | "lt" => Some(TriggerSide::Left),
        "right" | "r" | "rt" => Some(TriggerSide::Right),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_button_aliases() {
        assert!(matches!(parse_button("a"), Some(Button::A)));
        assert!(matches!(parse_button("DPAD_UP"), Some(Button::DPadUp)));
        assert!(matches!(parse_button("l3"), Some(Button::LeftThumb)));
        assert!(parse_button("unknown").is_none());
    }

    #[test]
    fn parses_axis_aliases() {
        assert!(matches!(parse_axis("LeftX"), Some(ThumbAxis::LeftX)));
        assert!(matches!(parse_axis("ry"), Some(ThumbAxis::RightY)));
        assert!(parse_axis("middle").is_none());
    }

    #[test]
    fn parses_trigger_aliases() {
        assert!(matches!(parse_trigger("left"), Some(TriggerSide::Left)));
        assert!(matches!(parse_trigger("rt"), Some(TriggerSide::Right)));
        assert!(parse_trigger("center").is_none());
    }
}
