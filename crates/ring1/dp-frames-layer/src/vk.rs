use std::ffi::{c_char, c_void};

pub type VkResult = i32;
pub const VK_SUCCESS: VkResult = 0;
pub const VK_ERROR_INITIALIZATION_FAILED: VkResult = -3;
pub const VK_ERROR_DEVICE_LOST: VkResult = -4;

pub const VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO: i32 = 47;
pub const VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO: i32 = 48;
pub const VK_LAYER_LINK_INFO: i32 = 0;

pub const LAYER_NEGOTIATE_INTERFACE_STRUCT: i32 = 1;
pub const CURRENT_LOADER_LAYER_INTERFACE_VERSION: u32 = 2;

/// Dispatchable handles are pointers to objects whose first word is the loader's dispatch table pointer.
pub type VkInstance = *mut c_void;
pub type VkPhysicalDevice = *mut c_void;
pub type VkDevice = *mut c_void;
pub type VkQueue = *mut c_void;

pub type PfnVoid = Option<unsafe extern "C" fn()>;
pub type PfnGetInstanceProcAddr = unsafe extern "C" fn(VkInstance, *const c_char) -> PfnVoid;
pub type PfnGetDeviceProcAddr = unsafe extern "C" fn(VkDevice, *const c_char) -> PfnVoid;
pub type PfnGetPhysicalDeviceProcAddr = unsafe extern "C" fn(VkInstance, *const c_char) -> PfnVoid;

pub type PfnCreateInstance = unsafe extern "C" fn(*const c_void, *const c_void, *mut VkInstance) -> VkResult;
pub type PfnDestroyInstance = unsafe extern "C" fn(VkInstance, *const c_void);
pub type PfnCreateDevice =
    unsafe extern "C" fn(VkPhysicalDevice, *const c_void, *const c_void, *mut VkDevice) -> VkResult;
pub type PfnDestroyDevice = unsafe extern "C" fn(VkDevice, *const c_void);
pub type PfnQueuePresent = unsafe extern "C" fn(VkQueue, *const VkPresentInfoKHR) -> VkResult;

/// Prefix shared by every extensible Vulkan structure.
#[repr(C)]
pub struct VkBaseStructure {
    pub s_type: i32,
    pub p_next: *const c_void,
}

#[repr(C)]
pub struct VkLayerInstanceLink {
    pub p_next: *mut VkLayerInstanceLink,
    pub next_get_instance_proc_addr: Option<PfnGetInstanceProcAddr>,
    pub next_get_physical_device_proc_addr: Option<PfnGetPhysicalDeviceProcAddr>,
}

#[repr(C)]
pub struct VkLayerDeviceLink {
    pub p_next: *mut VkLayerDeviceLink,
    pub next_get_instance_proc_addr: Option<PfnGetInstanceProcAddr>,
    pub next_get_device_proc_addr: Option<PfnGetDeviceProcAddr>,
}

/// `VkLayerInstanceCreateInfo`. The C union is two pointers wide; only `pLayerInfo` is read here.
#[repr(C)]
pub struct VkLayerInstanceCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub function: i32,
    pub p_layer_info: *mut VkLayerInstanceLink,
    pub union_tail: *mut c_void,
}

#[repr(C)]
pub struct VkLayerDeviceCreateInfo {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub function: i32,
    pub p_layer_info: *mut VkLayerDeviceLink,
    pub union_tail: *mut c_void,
}

#[repr(C)]
pub struct VkNegotiateLayerInterface {
    pub s_type: i32,
    pub p_next: *mut c_void,
    pub loader_layer_interface_version: u32,
    pub get_instance_proc_addr: Option<PfnGetInstanceProcAddr>,
    pub get_device_proc_addr: Option<PfnGetDeviceProcAddr>,
    pub get_physical_device_proc_addr: Option<PfnGetPhysicalDeviceProcAddr>,
}

#[repr(C)]
pub struct VkPresentInfoKHR {
    pub s_type: i32,
    pub p_next: *const c_void,
    pub wait_semaphore_count: u32,
    pub p_wait_semaphores: *const u64,
    pub swapchain_count: u32,
    pub p_swapchains: *const u64,
    pub p_image_indices: *const u32,
    pub p_results: *mut VkResult,
}
