use std::ffi::{c_char, c_void, CStr};
use std::mem::transmute;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

use crate::clock::now_ns;
use crate::recorder;
use crate::registry::Registry;
use crate::vk::*;

struct InstanceEntry {
    get_instance_proc_addr: PfnGetInstanceProcAddr,
    destroy_instance: Option<PfnDestroyInstance>,
    create_device: Option<PfnCreateDevice>,
}

struct DeviceEntry {
    get_device_proc_addr: PfnGetDeviceProcAddr,
    destroy_device: Option<PfnDestroyDevice>,
    queue_present: Option<PfnQueuePresent>,
}

static INSTANCES: Registry<InstanceEntry> = Registry::new();
static DEVICES: Registry<DeviceEntry> = Registry::new();

/// The loader keys its per-object dispatch by the first word of every dispatchable handle, and a
/// device and its queues share that word.
///
/// # Safety
/// `handle` must be null or a live Vulkan dispatchable handle, whose first word is readable. The Vulkan
/// valid-usage rules make the application (and so the loader) uphold that for every handle it passes down.
unsafe fn dispatch_key(handle: *mut c_void) -> usize {
    if handle.is_null() {
        0
    } else {
        *(handle as *const usize)
    }
}

/// # Safety
/// `name` must be null or a NUL-terminated string that stays valid for the call, as the loader guarantees for the
/// name argument of the proc-addr entry points.
unsafe fn name_is(name: *const c_char, expected: &CStr) -> bool {
    !name.is_null() && CStr::from_ptr(name) == expected
}

/// # Safety
/// `info` must point to a Vulkan create-info structure whose `pNext` chain is made of valid structures that each
/// start with `VkBaseStructure`. The loader builds that chain and keeps it alive for the whole create call. The
/// returned pointer is into that chain, so it is valid only for the same call.
unsafe fn find_instance_chain(info: *const c_void) -> *mut VkLayerInstanceCreateInfo {
    let mut node = (*(info as *const VkBaseStructure)).p_next as *mut VkLayerInstanceCreateInfo;
    while !node.is_null() {
        if (*node).s_type == VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO && (*node).function == VK_LAYER_LINK_INFO {
            return node;
        }
        node = (*node).p_next as *mut VkLayerInstanceCreateInfo;
    }
    ptr::null_mut()
}

/// # Safety
/// Same contract as `find_instance_chain`, for `VkDeviceCreateInfo`.
unsafe fn find_device_chain(info: *const c_void) -> *mut VkLayerDeviceCreateInfo {
    let mut node = (*(info as *const VkBaseStructure)).p_next as *mut VkLayerDeviceCreateInfo;
    while !node.is_null() {
        if (*node).s_type == VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO && (*node).function == VK_LAYER_LINK_INFO {
            return node;
        }
        node = (*node).p_next as *mut VkLayerDeviceCreateInfo;
    }
    ptr::null_mut()
}

/// The loader calls this first. Interface version 2 hands the layer its two entry points directly.
///
/// # Safety
/// `negotiate` must be null or point to a valid `VkNegotiateLayerInterface`.
#[no_mangle]
pub unsafe extern "C" fn vkNegotiateLoaderLayerInterfaceVersion(negotiate: *mut VkNegotiateLayerInterface) -> VkResult {
    if negotiate.is_null() || (*negotiate).s_type != LAYER_NEGOTIATE_INTERFACE_STRUCT {
        return VK_ERROR_INITIALIZATION_FAILED;
    }
    let n = &mut *negotiate;
    if n.loader_layer_interface_version < 2 {
        return VK_ERROR_INITIALIZATION_FAILED;
    }
    n.loader_layer_interface_version = CURRENT_LOADER_LAYER_INTERFACE_VERSION;
    n.get_instance_proc_addr = Some(get_instance_proc_addr);
    n.get_device_proc_addr = Some(get_device_proc_addr);
    n.get_physical_device_proc_addr = None;
    VK_SUCCESS
}

/// # Safety
/// Called only by the Vulkan loader, with `info` pointing to a valid `VkInstanceCreateInfo` and `out` to writable
/// storage for the new handle.
unsafe extern "C" fn create_instance(info: *const c_void, alloc: *const c_void, out: *mut VkInstance) -> VkResult {
    if info.is_null() {
        return VK_ERROR_INITIALIZATION_FAILED;
    }
    let chain = find_instance_chain(info);
    if chain.is_null() || (*chain).p_layer_info.is_null() {
        return VK_ERROR_INITIALIZATION_FAILED;
    }
    // SAFETY (this function): the Vulkan loader calls the layer entry points with valid arguments, so `info`, the
    // chain it carries and the link node are live for the call. The loader expects each layer to advance
    // `p_layer_info` to the next node before calling down, which is why it is written through here; the chain
    // is loader-owned memory and not shared with another thread during creation.
    let link = &*(*chain).p_layer_info;
    let Some(next_gipa) = link.next_get_instance_proc_addr else {
        return VK_ERROR_INITIALIZATION_FAILED;
    };
    (*chain).p_layer_info = link.p_next;

    let Some(next_create) = next_gipa(ptr::null_mut(), c"vkCreateInstance".as_ptr()) else {
        return VK_ERROR_INITIALIZATION_FAILED;
    };
    // SAFETY: `next_create` came from the next layer's `vkGetInstanceProcAddr` for the name "vkCreateInstance",
    // so it has the `PfnCreateInstance` signature. Both types are plain function pointers of the same size.
    let next_create: PfnCreateInstance = transmute(next_create);
    let result = next_create(info, alloc, out);
    if result == VK_SUCCESS {
        // SAFETY: on success the next layer has written a live instance handle through `out`, which the loader
        // gave us as valid storage. Each transmute turns the nullable pointer returned for that exact name into
        // the `Option` of its Vulkan signature; a null (extension absent) stays `None`.
        let instance = *out;
        INSTANCES.insert(
            dispatch_key(instance),
            InstanceEntry {
                get_instance_proc_addr: next_gipa,
                destroy_instance: transmute::<PfnVoid, Option<PfnDestroyInstance>>(next_gipa(
                    instance,
                    c"vkDestroyInstance".as_ptr(),
                )),
                create_device: transmute::<PfnVoid, Option<PfnCreateDevice>>(next_gipa(
                    instance,
                    c"vkCreateDevice".as_ptr(),
                )),
            },
        );
    }
    result
}

unsafe extern "C" fn destroy_instance(instance: VkInstance, alloc: *const c_void) {
    // SAFETY (this function): the loader passes the live instance being destroyed. The entry is removed before
    // the driver call so a handle reused by a new instance cannot be mistaken for this one.
    let key = dispatch_key(instance);
    let next = INSTANCES.get(key).and_then(|e| e.destroy_instance);
    INSTANCES.remove(key);
    if let Some(next) = next {
        next(instance, alloc);
    }
}

/// # Safety
/// Called only by the Vulkan loader, with a physical device of a layered instance, a valid `VkDeviceCreateInfo`
/// and writable storage for the new handle.
unsafe extern "C" fn create_device(
    physical_device: VkPhysicalDevice,
    info: *const c_void,
    alloc: *const c_void,
    out: *mut VkDevice,
) -> VkResult {
    if info.is_null() {
        return VK_ERROR_INITIALIZATION_FAILED;
    }
    let Some(next_create) = INSTANCES.get(dispatch_key(physical_device)).and_then(|e| e.create_device) else {
        return VK_ERROR_INITIALIZATION_FAILED;
    };
    let chain = find_device_chain(info);
    if chain.is_null() || (*chain).p_layer_info.is_null() {
        return VK_ERROR_INITIALIZATION_FAILED;
    }
    // SAFETY: as in `create_instance`: loader-supplied arguments are valid for the call and the link is advanced
    // before calling down. `next_create` was stored from this instance's own `vkCreateDevice` lookup, and the
    // physical device handle belongs to that instance, so its dispatch key finds the right entry.
    let link = &*(*chain).p_layer_info;
    let Some(next_gdpa) = link.next_get_device_proc_addr else {
        return VK_ERROR_INITIALIZATION_FAILED;
    };
    (*chain).p_layer_info = link.p_next;

    let result = next_create(physical_device, info, alloc, out);
    if result == VK_SUCCESS {
        // SAFETY: on success `out` holds the new live device. The transmutes follow the same rule as in
        // `create_instance`, with names looked up through this device's `next_gdpa`.
        let device = *out;
        DEVICES.insert(
            dispatch_key(device),
            DeviceEntry {
                get_device_proc_addr: next_gdpa,
                destroy_device: transmute::<PfnVoid, Option<PfnDestroyDevice>>(next_gdpa(
                    device,
                    c"vkDestroyDevice".as_ptr(),
                )),
                queue_present: transmute::<PfnVoid, Option<PfnQueuePresent>>(next_gdpa(
                    device,
                    c"vkQueuePresentKHR".as_ptr(),
                )),
            },
        );
        recorder::init();
    }
    result
}

unsafe extern "C" fn destroy_device(device: VkDevice, alloc: *const c_void) {
    // Removed before the driver call: once the device is destroyed its handle memory can be reused
    // by a new device, and removing afterwards would drop that one instead.
    let key = dispatch_key(device);
    let next = DEVICES.get(key).and_then(|e| e.destroy_device);
    DEVICES.remove(key);
    if let Some(next) = next {
        next(device, alloc);
    }
}

unsafe extern "C" fn queue_present(queue: VkQueue, info: *const VkPresentInfoKHR) -> VkResult {
    // SAFETY (this function): the loader passes a live queue and, per the Vulkan valid-usage rules, a
    // `VkPresentInfoKHR` whose swapchain array holds `swapchain_count` entries. Recording runs inside
    // `catch_unwind` so a panic can never unwind across the FFI boundary into the game, and the real present
    // always runs afterwards.
    let timestamp = now_ns();
    let Some(next) = DEVICES.get(dispatch_key(queue)).and_then(|e| e.queue_present) else {
        return VK_ERROR_DEVICE_LOST;
    };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !info.is_null() && (*info).swapchain_count > 0 && !(*info).p_swapchains.is_null() {
            let handle = *(*info).p_swapchains;
            recorder::record(timestamp, (handle ^ (handle >> 32)) as u32);
        }
    }));
    next(queue, info)
}

unsafe extern "C" fn get_device_proc_addr(device: VkDevice, name: *const c_char) -> PfnVoid {
    // SAFETY (this function): the loader passes a live device handle and a valid name. Each transmute widens a
    // layer function to the generic `PFN_vkVoidFunction`; the loader casts it back to the signature of the name
    // it asked for, which is the one the function was matched on.
    let entry = DEVICES.get(dispatch_key(device))?;
    if name_is(name, c"vkGetDeviceProcAddr") {
        return Some(transmute::<PfnGetDeviceProcAddr, unsafe extern "C" fn()>(get_device_proc_addr));
    }
    if name_is(name, c"vkDestroyDevice") {
        return Some(transmute::<PfnDestroyDevice, unsafe extern "C" fn()>(destroy_device));
    }
    if entry.queue_present.is_some() && name_is(name, c"vkQueuePresentKHR") {
        return Some(transmute::<PfnQueuePresent, unsafe extern "C" fn()>(queue_present));
    }
    (entry.get_device_proc_addr)(device, name)
}

unsafe extern "C" fn get_instance_proc_addr(instance: VkInstance, name: *const c_char) -> PfnVoid {
    // SAFETY (this function): same contract as `get_device_proc_addr`. `vkCreateInstance` is answered before the
    // instance lookup because it is queried with a null instance.
    if name_is(name, c"vkCreateInstance") {
        return Some(transmute::<PfnCreateInstance, unsafe extern "C" fn()>(create_instance));
    }
    let entry = INSTANCES.get(dispatch_key(instance))?;
    if name_is(name, c"vkGetInstanceProcAddr") {
        return Some(transmute::<PfnGetInstanceProcAddr, unsafe extern "C" fn()>(get_instance_proc_addr));
    }
    if name_is(name, c"vkDestroyInstance") {
        return Some(transmute::<PfnDestroyInstance, unsafe extern "C" fn()>(destroy_instance));
    }
    if name_is(name, c"vkCreateDevice") {
        return Some(transmute::<PfnCreateDevice, unsafe extern "C" fn()>(create_device));
    }
    if name_is(name, c"vkGetDeviceProcAddr") {
        return Some(transmute::<PfnGetDeviceProcAddr, unsafe extern "C" fn()>(get_device_proc_addr));
    }
    if name_is(name, c"vkDestroyDevice") {
        return Some(transmute::<PfnDestroyDevice, unsafe extern "C" fn()>(destroy_device));
    }
    if name_is(name, c"vkQueuePresentKHR") {
        return Some(transmute::<PfnQueuePresent, unsafe extern "C" fn()>(queue_present));
    }
    (entry.get_instance_proc_addr)(instance, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{read_until, temp_dir};
    use std::sync::atomic::{AtomicU32, Ordering};

    const INSTANCE_KEY: usize = 0xA1;
    const DEVICE_KEY: usize = 0xB2;

    static mut INSTANCE_OBJECT: usize = INSTANCE_KEY;
    static mut PHYSICAL_OBJECT: usize = INSTANCE_KEY;
    static mut DEVICE_OBJECT: usize = DEVICE_KEY;
    static NEXT_PRESENTS: AtomicU32 = AtomicU32::new(0);
    static NEXT_DESTROYS: AtomicU32 = AtomicU32::new(0);
    static SEEN_INFO_CHAIN_ADVANCED: AtomicU32 = AtomicU32::new(0);

    unsafe extern "C" fn next_create_instance(_: *const c_void, _: *const c_void, out: *mut VkInstance) -> VkResult {
        *out = ptr::addr_of_mut!(INSTANCE_OBJECT) as VkInstance;
        VK_SUCCESS
    }
    unsafe extern "C" fn next_destroy_instance(_: VkInstance, _: *const c_void) {
        NEXT_DESTROYS.fetch_add(1, Ordering::SeqCst);
    }
    unsafe extern "C" fn next_create_device(
        _: VkPhysicalDevice,
        info: *const c_void,
        _: *const c_void,
        out: *mut VkDevice,
    ) -> VkResult {
        let chain = (*(info as *const VkBaseStructure)).p_next as *const VkLayerDeviceCreateInfo;
        if (*chain).p_layer_info.is_null() {
            SEEN_INFO_CHAIN_ADVANCED.store(1, Ordering::SeqCst);
        }
        *out = ptr::addr_of_mut!(DEVICE_OBJECT) as VkDevice;
        VK_SUCCESS
    }
    unsafe extern "C" fn next_destroy_device(_: VkDevice, _: *const c_void) {
        NEXT_DESTROYS.fetch_add(1, Ordering::SeqCst);
    }
    unsafe extern "C" fn next_queue_present(_: VkQueue, _: *const VkPresentInfoKHR) -> VkResult {
        NEXT_PRESENTS.fetch_add(1, Ordering::SeqCst);
        VK_SUCCESS
    }
    unsafe extern "C" fn next_gipa(_: VkInstance, name: *const c_char) -> PfnVoid {
        if name_is(name, c"vkCreateInstance") {
            Some(transmute::<PfnCreateInstance, unsafe extern "C" fn()>(next_create_instance))
        } else if name_is(name, c"vkDestroyInstance") {
            Some(transmute::<PfnDestroyInstance, unsafe extern "C" fn()>(next_destroy_instance))
        } else if name_is(name, c"vkCreateDevice") {
            Some(transmute::<PfnCreateDevice, unsafe extern "C" fn()>(next_create_device))
        } else {
            None
        }
    }
    unsafe extern "C" fn next_gdpa(_: VkDevice, name: *const c_char) -> PfnVoid {
        if name_is(name, c"vkDestroyDevice") {
            Some(transmute::<PfnDestroyDevice, unsafe extern "C" fn()>(next_destroy_device))
        } else if name_is(name, c"vkQueuePresentKHR") {
            Some(transmute::<PfnQueuePresent, unsafe extern "C" fn()>(next_queue_present))
        } else {
            None
        }
    }

    unsafe fn resolve<T>(gipa: PfnGetInstanceProcAddr, instance: VkInstance, name: &CStr) -> T {
        let f = gipa(instance, name.as_ptr()).expect("layer exposes the function");
        transmute_copy_fn(f)
    }
    unsafe fn transmute_copy_fn<T>(f: unsafe extern "C" fn()) -> T {
        assert_eq!(std::mem::size_of::<T>(), std::mem::size_of_val(&f));
        std::mem::transmute_copy(&f)
    }

    #[test]
    fn negotiation_rejects_a_bad_struct_and_old_loaders() {
        unsafe {
            assert_eq!(vkNegotiateLoaderLayerInterfaceVersion(ptr::null_mut()), VK_ERROR_INITIALIZATION_FAILED);
            let mut n = VkNegotiateLayerInterface {
                s_type: 99,
                p_next: ptr::null_mut(),
                loader_layer_interface_version: 2,
                get_instance_proc_addr: None,
                get_device_proc_addr: None,
                get_physical_device_proc_addr: None,
            };
            assert_eq!(vkNegotiateLoaderLayerInterfaceVersion(&mut n), VK_ERROR_INITIALIZATION_FAILED);
            n.s_type = LAYER_NEGOTIATE_INTERFACE_STRUCT;
            n.loader_layer_interface_version = 1;
            assert_eq!(vkNegotiateLoaderLayerInterfaceVersion(&mut n), VK_ERROR_INITIALIZATION_FAILED);
        }
    }

    #[test]
    fn a_full_loader_sequence_records_presents_and_forwards_everything() {
        let dir = temp_dir("layer");
        std::env::set_var("DEADLOCK_PLUS_FRAMES_DIR", &dir);
        unsafe {
            let mut n = VkNegotiateLayerInterface {
                s_type: LAYER_NEGOTIATE_INTERFACE_STRUCT,
                p_next: ptr::null_mut(),
                loader_layer_interface_version: 7,
                get_instance_proc_addr: None,
                get_device_proc_addr: None,
                get_physical_device_proc_addr: None,
            };
            assert_eq!(vkNegotiateLoaderLayerInterfaceVersion(&mut n), VK_SUCCESS);
            assert_eq!(n.loader_layer_interface_version, 2);
            let gipa = n.get_instance_proc_addr.unwrap();
            let gdpa = n.get_device_proc_addr.unwrap();

            assert!(gipa(ptr::null_mut(), c"vkEnumerateInstanceVersion".as_ptr()).is_none());
            let create_instance_fn: PfnCreateInstance = resolve(gipa, ptr::null_mut(), c"vkCreateInstance");

            let mut link = VkLayerInstanceLink {
                p_next: ptr::null_mut(),
                next_get_instance_proc_addr: Some(next_gipa),
                next_get_physical_device_proc_addr: None,
            };
            let mut instance_chain = VkLayerInstanceCreateInfo {
                s_type: VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO,
                p_next: ptr::null(),
                function: VK_LAYER_LINK_INFO,
                p_layer_info: &mut link,
                union_tail: ptr::null_mut(),
            };
            let instance_info = VkBaseStructure { s_type: 1, p_next: &mut instance_chain as *mut _ as *const c_void };
            let mut instance: VkInstance = ptr::null_mut();
            assert_eq!(
                create_instance_fn(&instance_info as *const _ as *const c_void, ptr::null(), &mut instance),
                VK_SUCCESS
            );
            assert!(instance_chain.p_layer_info.is_null(), "the link must be advanced for the next layer");
            assert_eq!(dispatch_key(instance), INSTANCE_KEY);

            let create_device_fn: PfnCreateDevice = resolve(gipa, instance, c"vkCreateDevice");
            let mut device_link = VkLayerDeviceLink {
                p_next: ptr::null_mut(),
                next_get_instance_proc_addr: Some(next_gipa),
                next_get_device_proc_addr: Some(next_gdpa),
            };
            let mut device_chain = VkLayerDeviceCreateInfo {
                s_type: VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO,
                p_next: ptr::null(),
                function: VK_LAYER_LINK_INFO,
                p_layer_info: &mut device_link,
                union_tail: ptr::null_mut(),
            };
            let device_info = VkBaseStructure { s_type: 3, p_next: &mut device_chain as *mut _ as *const c_void };
            let mut device: VkDevice = ptr::null_mut();
            let physical = ptr::addr_of_mut!(PHYSICAL_OBJECT) as VkPhysicalDevice;
            assert_eq!(
                create_device_fn(physical, &device_info as *const _ as *const c_void, ptr::null(), &mut device),
                VK_SUCCESS
            );
            assert_eq!(SEEN_INFO_CHAIN_ADVANCED.load(Ordering::SeqCst), 1);

            let present: PfnQueuePresent = transmute_copy_fn(gdpa(device, c"vkQueuePresentKHR".as_ptr()).unwrap());
            assert!(gdpa(device, c"vkSomethingElse".as_ptr()).is_none(), "unknown names go to the next layer");
            let swapchains = [0x1_0000_0002u64];
            let present_info = VkPresentInfoKHR {
                s_type: 1000001001,
                p_next: ptr::null(),
                wait_semaphore_count: 0,
                p_wait_semaphores: ptr::null(),
                swapchain_count: 1,
                p_swapchains: swapchains.as_ptr(),
                p_image_indices: ptr::null(),
                p_results: ptr::null_mut(),
            };
            for _ in 0..50 {
                assert_eq!(present(device, &present_info), VK_SUCCESS);
            }
            assert_eq!(NEXT_PRESENTS.load(Ordering::SeqCst), 50);

            let records = read_until(&dir.join(format!("{}.dpf", std::process::id())), 50);
            assert_eq!(records.len(), 50);
            assert!(records.windows(2).all(|w| w[0].timestamp_ns <= w[1].timestamp_ns));
            assert!(records.iter().all(|r| r.swapchain == 0x1_0000_0002u64 as u32 ^ 1));

            let destroy_device_fn: PfnDestroyDevice = resolve(gipa, instance, c"vkDestroyDevice");
            destroy_device_fn(device, ptr::null());
            assert!(gdpa(device, c"vkQueuePresentKHR".as_ptr()).is_none(), "a destroyed device is forgotten");
            assert_eq!(present(device, &present_info), VK_ERROR_DEVICE_LOST);
            let destroy_instance_fn: PfnDestroyInstance = resolve(gipa, instance, c"vkDestroyInstance");
            destroy_instance_fn(instance, ptr::null());
            assert_eq!(NEXT_DESTROYS.load(Ordering::SeqCst), 2);
            assert!(gipa(instance, c"vkCreateDevice".as_ptr()).is_none());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
