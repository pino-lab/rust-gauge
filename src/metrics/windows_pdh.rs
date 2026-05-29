use windows_sys::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_MORE_DATA,
};

pub struct WindowsPdhCollector {
    engine: Option<PdhWildcardCounter>,
    gpu_usage: Option<f32>,
    npu_usage: Option<f32>,
}

impl WindowsPdhCollector {
    pub fn new() -> Self {
        Self {
            engine: PdhWildcardCounter::new(r"\GPU Engine(*)\Utilization Percentage"),
            gpu_usage: None,
            npu_usage: None,
        }
    }

    pub fn refresh(&mut self) {
        if let Some(engine) = &mut self.engine {
            engine.refresh();
            let samples = engine.samples();
            let npu_luids = compute_only_luids(&samples);
            let mut gpu_total = 0.0;
            let mut npu_total = 0.0;

            for sample in samples {
                if let Some(luid) = extract_luid(&sample.name) {
                    if npu_luids.iter().any(|candidate| candidate == &luid) {
                        npu_total += sample.value;
                    } else {
                        gpu_total += sample.value;
                    }
                } else {
                    gpu_total += sample.value;
                }
            }

            self.gpu_usage = Some(gpu_total.clamp(0.0, 100.0) as f32);
            self.npu_usage = if npu_luids.is_empty() {
                None
            } else {
                Some(npu_total.clamp(0.0, 100.0) as f32)
            };
        }
    }

    pub fn gpu_usage_percent(&mut self) -> Option<f32> {
        self.gpu_usage
    }

    pub fn npu_usage_percent(&mut self) -> Option<f32> {
        self.npu_usage
    }
}

struct PdhWildcardCounter {
    query: isize,
    counter: isize,
    primed: bool,
}

impl PdhWildcardCounter {
    fn new(path: &str) -> Option<Self> {
        let mut query = 0;
        let status = unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) };
        if status != 0 {
            return None;
        }

        let mut counter = 0;
        let wide_path = to_wide(path);
        let status = unsafe { PdhAddEnglishCounterW(query, wide_path.as_ptr(), 0, &mut counter) };
        if status != 0 {
            unsafe {
                PdhCloseQuery(query);
            }
            return None;
        }

        Some(Self {
            query,
            counter,
            primed: false,
        })
    }

    fn refresh(&mut self) {
        unsafe {
            PdhCollectQueryData(self.query);
        }
        self.primed = true;
    }

    fn samples(&mut self) -> Vec<EngineSample> {
        if !self.primed {
            return Vec::new();
        }

        let mut buffer_size = 0;
        let mut item_count = 0;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                std::ptr::null_mut(),
            )
        };

        if status != PDH_MORE_DATA || buffer_size == 0 || item_count == 0 {
            return Vec::new();
        }

        let item_capacity = (buffer_size as usize)
            .div_ceil(std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>())
            .max(item_count as usize);
        let mut buffer = Vec::<PDH_FMT_COUNTERVALUE_ITEM_W>::with_capacity(item_capacity);
        let item_buffer = buffer.as_mut_ptr();
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                item_buffer,
            )
        };

        if status != 0 {
            return Vec::new();
        }

        let items = unsafe { std::slice::from_raw_parts(item_buffer, item_count as usize) };
        items
            .iter()
            .filter_map(|item| {
                let value = unsafe { item.FmtValue.Anonymous.doubleValue };
                if !value.is_finite() || value < 0.0 {
                    return None;
                }

                Some(EngineSample {
                    name: unsafe { wide_ptr_to_string(item.szName) },
                    value,
                })
            })
            .collect()
    }
}

impl Drop for PdhWildcardCounter {
    fn drop(&mut self) {
        unsafe {
            PdhCloseQuery(self.query);
        }
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[derive(Debug)]
struct EngineSample {
    name: String,
    value: f64,
}

fn compute_only_luids(samples: &[EngineSample]) -> Vec<String> {
    let mut groups = Vec::<LuidGroup>::new();

    for sample in samples {
        let Some(luid) = extract_luid(&sample.name) else {
            continue;
        };

        let engine_type = extract_engine_type(&sample.name).unwrap_or_default();
        if let Some(group) = groups.iter_mut().find(|group| group.luid == luid) {
            group.add_engine_type(&engine_type);
        } else {
            let mut group = LuidGroup {
                luid,
                has_compute: false,
                has_non_compute: false,
            };
            group.add_engine_type(&engine_type);
            groups.push(group);
        }
    }

    groups
        .into_iter()
        .filter(|group| group.has_compute && !group.has_non_compute)
        .map(|group| group.luid)
        .collect()
}

struct LuidGroup {
    luid: String,
    has_compute: bool,
    has_non_compute: bool,
}

impl LuidGroup {
    fn add_engine_type(&mut self, engine_type: &str) {
        if engine_type.starts_with("compute") {
            self.has_compute = true;
        } else if !engine_type.is_empty() {
            self.has_non_compute = true;
        }
    }
}

fn extract_luid(name: &str) -> Option<String> {
    let start = name.find("_luid_")? + "_luid_".len();
    let end = name[start..].find("_phys_")? + start;
    Some(name[start..end].to_string())
}

fn extract_engine_type(name: &str) -> Option<String> {
    let start = name.find("_engtype_")? + "_engtype_".len();
    Some(name[start..].to_ascii_lowercase())
}

unsafe fn wide_ptr_to_string(value: *const u16) -> String {
    if value.is_null() {
        return String::new();
    }

    let mut len = 0;
    while *value.add(len) != 0 {
        len += 1;
    }

    String::from_utf16_lossy(std::slice::from_raw_parts(value, len))
}
