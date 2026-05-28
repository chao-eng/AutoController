use libloading::{Library, Symbol};
use std::path::PathBuf;
use std::sync::Arc;

#[allow(non_camel_case_types)]
pub type PVIGEM_CLIENT = *mut std::ffi::c_void;
#[allow(non_camel_case_types)]
pub type PVIGEM_TARGET = *mut std::ffi::c_void;
#[allow(non_camel_case_types)]
pub type VIGEM_ERROR = u32;

pub const VIGEM_ERROR_NONE: VIGEM_ERROR = 0;
pub const VIGEM_ERROR_BUS_NOT_FOUND: VIGEM_ERROR = 1;
pub const VIGEM_ERROR_BUS_ACCESS_FAILED: VIGEM_ERROR = 2;
pub const VIGEM_ERROR_NO_FREE_SLOT: VIGEM_ERROR = 3;
pub const VIGEM_ERROR_INVALID_TARGET: VIGEM_ERROR = 4;
pub const VIGEM_ERROR_REMOVAL_FAILED: VIGEM_ERROR = 5;
pub const VIGEM_ERROR_ALREADY_CONNECTED: VIGEM_ERROR = 6;
pub const VIGEM_ERROR_TARGET_UNINITIALIZED: VIGEM_ERROR = 7;
pub const VIGEM_ERROR_NOT_PLUGGED_IN: VIGEM_ERROR = 8;

pub fn vigem_success(error: VIGEM_ERROR) -> bool {
    (error as i32) >= 0
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct XusbReport {
    pub w_buttons: u16,
    pub b_left_trigger: u8,
    pub b_right_trigger: u8,
    pub s_thumb_lx: i16,
    pub s_thumb_ly: i16,
    pub s_thumb_rx: i16,
    pub s_thumb_ry: i16,
}

type FnAlloc = unsafe extern "system" fn() -> PVIGEM_CLIENT;
type FnConnect = unsafe extern "system" fn(PVIGEM_CLIENT) -> VIGEM_ERROR;
type FnDisconnect = unsafe extern "system" fn(PVIGEM_CLIENT);
type FnFree = unsafe extern "system" fn(PVIGEM_CLIENT);
type FnTargetX360Alloc = unsafe extern "system" fn() -> PVIGEM_TARGET;
type FnTargetAdd = unsafe extern "system" fn(PVIGEM_CLIENT, PVIGEM_TARGET) -> VIGEM_ERROR;
type FnTargetRemove = unsafe extern "system" fn(PVIGEM_CLIENT, PVIGEM_TARGET) -> VIGEM_ERROR;
type FnTargetFree = unsafe extern "system" fn(PVIGEM_TARGET);
type FnTargetX360Update = unsafe extern "system" fn(PVIGEM_CLIENT, PVIGEM_TARGET, XusbReport) -> VIGEM_ERROR;
type FnTargetGetIndex = unsafe extern "system" fn(PVIGEM_TARGET) -> u32;

pub struct ViGEmBindings {
    _dll: Library,
    pub vigem_alloc: FnAlloc,
    pub vigem_connect: FnConnect,
    pub vigem_disconnect: FnDisconnect,
    pub vigem_free: FnFree,
    pub vigem_target_x360_alloc: FnTargetX360Alloc,
    pub vigem_target_add: FnTargetAdd,
    pub vigem_target_remove: FnTargetRemove,
    pub vigem_target_free: FnTargetFree,
    pub vigem_target_x360_update: FnTargetX360Update,
    pub vigem_target_get_index: FnTargetGetIndex,
}

impl ViGEmBindings {
    unsafe fn load_sym<T: Copy>(dll: &Library, name: &[u8]) -> Result<T, String> {
        let sym: Symbol<T> = dll
            .get(name)
            .map_err(|e| format!("无法加载符号 {}: {}", String::from_utf8_lossy(name), e))?;
        Ok(*sym.into_raw())
    }

    pub fn load() -> Result<Self, String> {
        let dll_path = Self::find_dll();
        let dll = unsafe {
            match dll_path {
                Some(path) => Library::new(&path),
                None => Library::new("ViGEmClient.dll"),
            }
            .map_err(|e| format!("无法加载 ViGEmClient.dll: {}", e))?
        };

        unsafe {
            let vigem_alloc = Self::load_sym::<FnAlloc>(&dll, b"vigem_alloc\0")?;
            let vigem_connect = Self::load_sym::<FnConnect>(&dll, b"vigem_connect\0")?;
            let vigem_disconnect = Self::load_sym::<FnDisconnect>(&dll, b"vigem_disconnect\0")?;
            let vigem_free = Self::load_sym::<FnFree>(&dll, b"vigem_free\0")?;
            let vigem_target_x360_alloc =
                Self::load_sym::<FnTargetX360Alloc>(&dll, b"vigem_target_x360_alloc\0")?;
            let vigem_target_add = Self::load_sym::<FnTargetAdd>(&dll, b"vigem_target_add\0")?;
            let vigem_target_remove =
                Self::load_sym::<FnTargetRemove>(&dll, b"vigem_target_remove\0")?;
            let vigem_target_free =
                Self::load_sym::<FnTargetFree>(&dll, b"vigem_target_free\0")?;
            let vigem_target_x360_update =
                Self::load_sym::<FnTargetX360Update>(&dll, b"vigem_target_x360_update\0")?;
            let vigem_target_get_index =
                Self::load_sym::<FnTargetGetIndex>(&dll, b"vigem_target_get_index\0")?;

            Ok(Self {
                _dll: dll,
                vigem_alloc,
                vigem_connect,
                vigem_disconnect,
                vigem_free,
                vigem_target_x360_alloc,
                vigem_target_add,
                vigem_target_remove,
                vigem_target_free,
                vigem_target_x360_update,
                vigem_target_get_index,
            })
        }
    }

    fn find_dll() -> Option<PathBuf> {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                let dll = dir.join("ViGEmClient.dll");
                if dll.exists() {
                    return Some(dll);
                }
            }
        }

        let common_paths = [
            PathBuf::from(r"C:\Program Files\Nefarius\ViGEmBus\ViGEmClient.dll"),
            PathBuf::from(r"C:\Program Files (x86)\Nefarius\ViGEmBus\ViGEmClient.dll"),
        ];

        for path in &common_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }

        None
    }
}

pub struct ViGEmClient {
    bindings: Arc<ViGEmBindings>,
    client: PVIGEM_CLIENT,
    connected: bool,
}

unsafe impl Send for ViGEmClient {}
unsafe impl Sync for ViGEmClient {}

impl ViGEmClient {
    pub fn new(bindings: Arc<ViGEmBindings>) -> Result<Self, (u32, String)> {
        let client = unsafe { (bindings.vigem_alloc)() };
        if client.is_null() {
            return Err((0, "vigem_alloc 返回空指针".to_string()));
        }

        let error = unsafe { (bindings.vigem_connect)(client) };
        if !vigem_success(error) {
            unsafe { (bindings.vigem_free)(client) };
            return Err((error, match error {
                VIGEM_ERROR_BUS_NOT_FOUND => {
                    "ViGEmBus 驱动未找到，请先安装 ViGEmBus 驱动".to_string()
                }
                VIGEM_ERROR_BUS_ACCESS_FAILED => {
                    "无法访问 ViGEmBus 驱动，请尝试以管理员身份运行".to_string()
                }
                _ => format!("vigem_connect 失败，错误码: {} (0x{:08X})", error, error),
            }));
        }

        tracing::info!("已成功连接到 ViGEmBus 驱动");

        Ok(Self {
            bindings,
            client,
            connected: true,
        })
    }

    pub fn create_x360_target(&self) -> Result<PVIGEM_TARGET, String> {
        let target = unsafe { (self.bindings.vigem_target_x360_alloc)() };
        if target.is_null() {
            return Err("vigem_target_x360_alloc 返回空指针".to_string());
        }

        let error = unsafe { (self.bindings.vigem_target_add)(self.client, target) };
        if !vigem_success(error) {
            unsafe { (self.bindings.vigem_target_free)(target) };
            return Err(match error {
                VIGEM_ERROR_NO_FREE_SLOT => "没有可用的设备槽位".to_string(),
                VIGEM_ERROR_ALREADY_CONNECTED => "设备已连接".to_string(),
                _ => format!("vigem_target_add 失败，错误码: {} (0x{:08X})", error, error),
            });
        }

        let index = unsafe { (self.bindings.vigem_target_get_index)(target) };
        tracing::info!(user_index = index, "Xbox 360 虚拟手柄已创建并连接");

        Ok(target)
    }


    pub fn remove_target(&self, target: PVIGEM_TARGET) -> Result<(), String> {
        let error = unsafe { (self.bindings.vigem_target_remove)(self.client, target) };
        if !vigem_success(error) {
            return Err(format!("vigem_target_remove 失败，错误码: {} (0x{:08X})", error, error));
        }
        unsafe { (self.bindings.vigem_target_free)(target) };
        Ok(())
    }

    pub fn update_x360(&self, target: PVIGEM_TARGET, report: XusbReport) -> Result<(), String> {
        let error = unsafe { (self.bindings.vigem_target_x360_update)(self.client, target, report) };
        if !vigem_success(error) {
            return Err(format!("vigem_target_x360_update 失败，错误码: {} (0x{:08X})", error, error));
        }
        Ok(())
    }

    pub fn get_target_index(&self, target: PVIGEM_TARGET) -> u32 {
        unsafe { (self.bindings.vigem_target_get_index)(target) }
    }
}

impl Drop for ViGEmClient {
    fn drop(&mut self) {
        if self.connected && !self.client.is_null() {
            unsafe {
                (self.bindings.vigem_disconnect)(self.client);
                (self.bindings.vigem_free)(self.client);
            }
            tracing::info!("已断开 ViGEmBus 连接");
        }
    }
}
