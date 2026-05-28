use libloading::{Library, Symbol};

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct XinputGamepad {
    pub w_buttons: u16,
    pub b_left_trigger: u8,
    pub b_right_trigger: u8,
    pub s_thumb_lx: i16,
    pub s_thumb_ly: i16,
    pub s_thumb_rx: i16,
    pub s_thumb_ry: i16,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct XinputState {
    pub dw_packet_number: u32,
    pub gamepad: XinputGamepad,
}

type FnXInputGetState = unsafe extern "system" fn(u32, *mut XinputState) -> u32;

pub struct XInputBindings {
    _dll: Library,
    xinput_get_state: FnXInputGetState,
}

impl XInputBindings {
    pub fn load() -> Result<Self, String> {
        let dll_paths = [
            "xinput1_4.dll".to_string(),
            "xinput1_3.dll".to_string(),
            "xinput9_1_0.dll".to_string(),
            r"C:\Windows\System32\xinput1_4.dll".to_string(),
            r"C:\Windows\System32\xinput1_3.dll".to_string(),
            r"C:\Windows\System32\xinput9_1_0.dll".to_string(),
            r"C:\Windows\SysWOW64\xinput1_4.dll".to_string(),
            r"C:\Windows\SysWOW64\xinput1_3.dll".to_string(),
            r"C:\Windows\SysWOW64\xinput9_1_0.dll".to_string(),
        ];
        let mut loaded_dll = None;
        let mut errors = Vec::new();

        for path in &dll_paths {
            match unsafe { Library::new(path) } {
                Ok(dll) => {
                    loaded_dll = Some(dll);
                    break;
                }
                Err(e) => {
                    errors.push(format!("{}: {}", path, e));
                }
            }
        }

        let dll = loaded_dll.ok_or_else(|| {
            format!(
                "无法加载任何 XInput DLL！加载错误详情：\n- {}",
                errors.join("\n- ")
            )
        })?;
        
        let xinput_get_state = unsafe {
            let sym: Symbol<FnXInputGetState> = dll.get(b"XInputGetState\0")
                .map_err(|e| format!("无法找到 XInputGetState 符号: {}", e))?;
            *sym.into_raw()
        };

        Ok(Self {
            _dll: dll,
            xinput_get_state,
        })
    }

    pub fn get_state(&self, user_index: u32) -> Result<XinputState, u32> {
        let mut state = XinputState::default();
        let result = unsafe { (self.xinput_get_state)(user_index, &mut state) };
        if result == 0 {
            Ok(state)
        } else {
            Err(result)
        }
    }
}
