/*
 * Copyright (C) 2026 Project EVA / CosFix
 *
 * Transsion Hardware Subsystem Bridge for MediaTek & Transsion Platforms
 * (Tecno Camon 30 / Helio G99 / MT6789 / Dimensity / Infinix / Itel)
 */


pub const NODE_BATTERY_CAPACITY: &str = "/sys/class/power_supply/battery/capacity";
pub const NODE_BATTERY_VOLTAGE: &str = "/sys/class/power_supply/battery/voltage_now";
pub const NODE_BATTERY_CURRENT: &str = "/sys/class/power_supply/battery/current_now";
pub const NODE_BATTERY_TEMP: &str = "/sys/class/power_supply/battery/temp";
pub const NODE_BATTERY_STATUS: &str = "/sys/class/power_supply/battery/status";
pub const NODE_BATTERY_CHARGE_FULL: &str = "/sys/class/power_supply/battery/charge_full";
pub const NODE_BATTERY_CHARGE_COUNTER: &str = "/sys/class/power_supply/battery/charge_counter";
pub const NODE_BATTERY_CYCLE_COUNT: &str = "/sys/class/power_supply/battery/cycle_count";

pub const NODE_CHARGER_ONLINE: &str = "/sys/class/power_supply/charger/online";
pub const NODE_CHARGER_USB_TYPE: &str = "/sys/class/power_supply/charger/usb_type";

pub const NODE_TRAN_ADAPTER_CAPACITY: &str = "/sys/devices/platform/charger/adapter_capacity";
pub const NODE_TRAN_TA_POWER: &str = "/sys/class/power_supply/tran-auto-test/ta_power";
pub const NODE_TRAN_CHARGING_MODE: &str = "/sys/devices/platform/charger/Charging_mode";
pub const NODE_TRAN_CHARGER_TYPE: &str = "/sys/devices/platform/charger/Charger_Type";
pub const NODE_TRAN_PD_TYPE: &str = "/sys/devices/platform/charger/pd_type";
pub const NODE_TRAN_PUMP_EXPRESS: &str = "/sys/devices/platform/charger/Pump_Express";
pub const NODE_TRAN_FAST_CHG: &str = "/sys/devices/platform/charger/fast_chg_indicator";
pub const NODE_TRAN_ADC_VBUS: &str = "/sys/devices/platform/charger/ADC_Charger_Voltage";
pub const NODE_TRAN_ADC_IBUS: &str = "/sys/devices/platform/charger/ADC_Charging_Current";
pub const NODE_TRAN_PRIMARY_DVCHG: &str = "/sys/class/power_supply/primary_dvchg/present";
pub const NODE_TRAN_CHG_TYPE: &str = "/sys/class/power_supply/tran-auto-test/chg_type";
pub const NODE_TRAN_CHG_STATUS: &str = "/sys/class/power_supply/tran-auto-test/chg_status";

pub const NODE_TRAN_BYPASS_CHARGER: &str = "/sys/devices/platform/charger/bypass_charger";
pub const NODE_TRAN_HEALTH_PERCENT: &str = "/sys/devices/platform/charger/battery_health_percent";
pub const NODE_TRAN_BATTERY_CYCLE: &str = "/sys/devices/platform/charger/tran_battery_cycle";
pub const NODE_TRAN_SC_TUISOC: &str = "/sys/devices/platform/charger/sc_tuisoc";
pub const NODE_TRAN_RUST_DETECT: &str = "/sys/devices/platform/charger/Rust_detect";
pub const NODE_TRAN_CC1_STATUS: &str = "/sys/class/power_supply/tran-auto-test/cc1_status";
pub const NODE_TRAN_CC2_STATUS: &str = "/sys/class/power_supply/tran-auto-test/cc2_status";
pub const NODE_TRAN_RUN_PATH: &str = "/sys/class/power_supply/tran-auto-test/run_path";
pub const NODE_TRAN_HIGH_VOLT_ENABLE: &str = "/sys/devices/platform/charger/High_voltage_chg_enable";

const SYNTHETIC_DECIMAL_MIN_CENTI: i32 = 5;
const SYNTHETIC_DECIMAL_SEED_MAX_CENTI: i32 = 50;
const SYNTHETIC_DECIMAL_MAX_CENTI: i32 = 99;
const SYNTHETIC_DECIMAL_STEP_CENTI: i32 = 1;
const SYNTHETIC_DECIMAL_MAX_STEP_CENTI: i32 = 3;

pub const UI_SOC_DECIMAL_PATHS: &[&str] = &[
    "/sys/class/power_supply/tran-auto-test/ui_soc",
    "/proc/ui_soc_decimal",
    "/sys/class/power_supply/bms/soc_decimal",
    "/sys/class/qcom-battery/soc_decimal",
];

pub const UI_SOC_DECIMAL_RATE_PATHS: &[&str] = &[
    "/sys/class/power_supply/bms/soc_decimal_rate",
    "/sys/class/qcom-battery/soc_decimal_rate",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalSocState {
    pub capacity: i32,
    pub target: i32,
    pub random: u32,
}

impl Default for DecimalSocState {
    fn default() -> Self {
        Self {
            capacity: -1,
            target: 0,
            random: 0,
        }
    }
}

static DECIMAL_STATE: parking_lot::Mutex<DecimalSocState> =
    parking_lot::Mutex::new(DecimalSocState {
        capacity: -1,
        target: 0,
        random: 0,
    });

impl DecimalSocState {
    pub fn next_random(&mut self) -> u32 {
        if self.random == 0 {
            self.random = 0x6d2b_79f5;
        }
        self.random = self
            .random
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.random
    }
}

pub fn normalize_decimal_seed_offset(raw: i32, capacity: i32) -> Option<i32> {
    if raw <= 0 {
        return None;
    }

    let capacity = capacity.clamp(0, 100);
    let base = capacity * 100;
    let normalized = if raw <= 100 {
        raw * 100
    } else if raw <= 10000 {
        raw
    } else {
        raw / 100
    };
    let offset = normalized - base;
    if (SYNTHETIC_DECIMAL_MIN_CENTI..=SYNTHETIC_DECIMAL_MAX_CENTI).contains(&offset) {
        Some(offset)
    } else {
        None
    }
}

pub fn next_synthetic_decimal_seed(
    state: &mut DecimalSocState,
    capacity: i32,
    cached_seed: i32,
) -> i32 {
    normalize_decimal_seed_offset(cached_seed, capacity).unwrap_or_else(|| {
        SYNTHETIC_DECIMAL_MIN_CENTI
            + (state.next_random()
                % ((SYNTHETIC_DECIMAL_SEED_MAX_CENTI - SYNTHETIC_DECIMAL_MIN_CENTI + 1) as u32))
                as i32
    })
}

pub fn normalize_decimal_rate(rate: i32, wattage: i32) -> i32 {
    if rate > 0 {
        if rate > 10000 {
            ((rate / 100) / 4).clamp(
                SYNTHETIC_DECIMAL_STEP_CENTI,
                SYNTHETIC_DECIMAL_MAX_STEP_CENTI,
            )
        } else {
            (rate / 4).clamp(
                SYNTHETIC_DECIMAL_STEP_CENTI,
                SYNTHETIC_DECIMAL_MAX_STEP_CENTI,
            )
        }
    } else if wattage >= 60 {
        SYNTHETIC_DECIMAL_MAX_STEP_CENTI
    } else if wattage >= 30 {
        2
    } else {
        SYNTHETIC_DECIMAL_STEP_CENTI
    }
}

pub fn synthetic_decimal_soc_pair(
    state: &mut DecimalSocState,
    capacity: i32,
    cached_seed: i32,
    cached_rate: i32,
    wattage: i32,
) -> (i32, i32) {
    let capacity = capacity.clamp(0, 100);
    if capacity >= 100 {
        state.capacity = capacity;
        state.target = 10000;
        return (10000, 10000);
    }

    let base = capacity * 100;
    let min = base + SYNTHETIC_DECIMAL_MIN_CENTI;
    let max = base + SYNTHETIC_DECIMAL_MAX_CENTI;
    let step = normalize_decimal_rate(cached_rate, wattage).min(max - min);

    if state.capacity != capacity {
        state.capacity = capacity;
        state.target = base + next_synthetic_decimal_seed(state, capacity, cached_seed);
    }

    let start = state.target.clamp(min, max);
    let seed = normalize_decimal_seed_offset(cached_seed, capacity)
        .map(|offset| base + offset)
        .unwrap_or(start);
    let end = (start + step).max(seed).clamp(start, max);
    state.target = end;
    (start, end)
}

pub fn read_int_any(paths: &[&str]) -> i32 {
    for path in paths {
        let val = fast_read_int(path, -1);
        if val >= 0 {
            return val;
        }
    }
    0
}

pub fn parse_toggle_flag(extra: &str) -> bool {
    let trimmed = extra.trim();
    if trimmed.contains("switch=0")
        || trimmed == "0"
        || trimmed.contains("false")
        || trimmed.contains("disable")
        || trimmed == "off"
    {
        return false;
    }
    trimmed.contains("switch=1")
        || trimmed == "1"
        || trimmed.contains("true")
        || trimmed.contains("enable")
        || trimmed == "on"
}

pub fn fast_read_int(path: &str, default_val: i32) -> i32 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| {
            let trimmed = s.trim();
            if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
                let clean_hex: String = hex.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
                i32::from_str_radix(&clean_hex, 16).ok()
            } else {
                let token = trimmed.split_whitespace().next().unwrap_or(trimmed);
                token.parse::<i32>().ok().or_else(|| {
                    let digits: String = token
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '-')
                        .collect();
                    digits.parse::<i32>().ok()
                })
            }
        })
        .unwrap_or(default_val)
}

pub fn fast_read_string(path: &str, default_val: &str) -> String {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_val.to_string())
}

pub fn fast_write_string(path: &str, val: &str) -> bool {
    std::fs::write(path, val).is_ok()
}

pub fn fast_write_int(path: &str, val: i32) -> bool {
    std::fs::write(path, val.to_string()).is_ok()
}

#[cfg(target_os = "android")]
pub fn get_system_property(key: &str) -> Option<String> {
    use std::ffi::{CStr, CString};
    let c_key = CString::new(key).ok()?;
    let mut buf = [0 as libc::c_char; 92]; // PROP_VALUE_MAX = 92
    extern "C" {
        fn __system_property_get(
            name: *const libc::c_char,
            value: *mut libc::c_char,
        ) -> libc::c_int;
    }
    let len = unsafe { __system_property_get(c_key.as_ptr(), buf.as_mut_ptr()) };
    if len > 0 {
        let c_str = unsafe { CStr::from_ptr(buf.as_ptr()) };
        c_str
            .to_str()
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

#[cfg(not(target_os = "android"))]
pub fn get_system_property(_key: &str) -> Option<String> {
    None
}

#[derive(Debug, Clone, Default)]
pub struct TranssionState {
    pub online: bool,
    pub is_pc_usb: bool,
    pub wattage: i32,
    pub is_fast: bool,
    pub is_supervooc: bool,
    pub tech: i32,      // 0 = Normal, 1 = QC/PD, 2 = VOOC, 3 = SuperVOOC
    pub fast_type: i32, // 0 = Normal, 1 = VOOC, 2 = SuperVOOC 65W+
    pub ui_icon: i32,   // 0 = none, 1 = normal bolt, 2 = VOOC, 3 = SuperVOOC animation
    pub sup_speed: bool,
    pub battery_level: i32,
    pub voltage_mv: i32,
    pub current_ma: i32,
    pub temp_tenths: i32,
    pub status: String,
    pub soh: i32,
    pub cycle_count: i32,
    pub bypass_active: bool,
    pub limit_soc: i32,
    pub charge_counter_mah: i32,
    pub charge_full_mah: i32,
    pub decimal_seed: i32,
    pub decimal_rate: i32,
    pub usb_status: i32,
}

static CACHED_STATE: parking_lot::RwLock<Option<TranssionState>> =
    parking_lot::RwLock::new(None);

impl TranssionState {
    pub fn current() -> Self {
        {
            let guard = CACHED_STATE.read();
            if let Some(st) = guard.as_ref() {
                return st.clone();
            }
        }
        let fresh = Self::resolve();
        *CACHED_STATE.write() = Some(fresh.clone());
        fresh
    }

    pub fn update_cache(snapshot: Self) {
        *CACHED_STATE.write() = Some(snapshot);
    }

    fn normalize_voltage(raw: i32) -> i32 {
        if raw > 100_000 {
            raw / 1000
        } else {
            raw
        }
    }

    fn normalize_current(raw: i32) -> i32 {
        if raw.abs() > 10_000 {
            raw / 1000
        } else {
            raw
        }
    }

    fn resolve_cycle_count() -> i32 {
        let tran_cyc = fast_read_int("/sys/class/power_supply/tran-auto-test/bat_cycle", -1);
        if tran_cyc >= 0 {
            return tran_cyc;
        }
        let cyc = fast_read_int(NODE_TRAN_BATTERY_CYCLE, -1);
        if cyc >= 0 {
            return cyc;
        }
        fast_read_int(NODE_BATTERY_CYCLE_COUNT, 0)
    }

    fn check_property_wattage() -> Option<i32> {
        for prop in &[
            "persist.sys.charge_wattage",
            "persist.sys.oplus.charge_wattage",
            "persist.sys.rianixia.charge-wattage",
        ] {
            if let Some(val) = get_system_property(prop) {
                if let Ok(w) = val.parse::<i32>() {
                    if w > 0 {
                        return Some(w);
                    }
                }
            }
        }
        None
    }

    pub fn resolve() -> Self {
        let mut st = TranssionState::default();

        let raw_online = fast_read_int(NODE_CHARGER_ONLINE, 0) != 0;
        let batt_status = fast_read_string(NODE_BATTERY_STATUS, "Discharging");
        st.online = raw_online || batt_status == "Charging" || batt_status == "Full";
        st.status = if st.online {
            batt_status
        } else {
            "Discharging".to_string()
        };

        if !st.online {
            st.battery_level = fast_read_int(NODE_BATTERY_CAPACITY, 50);
            st.voltage_mv = Self::normalize_voltage(fast_read_int(NODE_BATTERY_VOLTAGE, 3800000));
            st.current_ma = Self::normalize_current(fast_read_int(NODE_BATTERY_CURRENT, 0));
            st.temp_tenths = fast_read_int(NODE_BATTERY_TEMP, 280);
            st.soh = fast_read_int(NODE_TRAN_HEALTH_PERCENT, 100).clamp(1, 100);
            st.cycle_count = Self::resolve_cycle_count();
            return st;
        }

        // Check if connection is a PC USB port (SDP) vs Wall Charger
        let usb_type_str = fast_read_string(NODE_CHARGER_USB_TYPE, "");
        let charger_type_num = fast_read_int(NODE_TRAN_CHARGER_TYPE, -1);
        let chg_type = fast_read_string(NODE_TRAN_CHG_TYPE, "");

        // SDP is strictly when usb_type has [SDP] or tran-auto-test chg_type is SDP,
        // or charger_type_num is explicitly 1 (MTK STANDARD_HOST).
        // NEVER treat charger_type_num == 0 as SDP (0 is CHARGER_UNKNOWN).
        let is_sdp = usb_type_str.contains("[SDP]")
            || chg_type == "SDP"
            || chg_type.contains("SDP")
            || charger_type_num == 1;

        // Transsion Hardware Sysfs Inspection
        let adapter_cap = fast_read_int(NODE_TRAN_ADAPTER_CAPACITY, 0);
        let ta_pwr = fast_read_int(NODE_TRAN_TA_POWER, 0);
        let pump_express = fast_read_int(NODE_TRAN_PUMP_EXPRESS, 0);
        let primary_dvchg = fast_read_int(NODE_TRAN_PRIMARY_DVCHG, 0);
        let fast_indicator = fast_read_int(NODE_TRAN_FAST_CHG, 0);
        let vbus_mv = fast_read_int(NODE_TRAN_ADC_VBUS, 0);
        let ibus_ma = fast_read_int(NODE_TRAN_ADC_IBUS, 0);

        let chg_mode = fast_read_string(NODE_TRAN_CHARGING_MODE, "");

        // Hardware CC pin resistor comparator (Type-C 3.0A advertising for 15W - 100W+ fast chargers)
        // Measured directly by analog PHY comparator with 0ms delay at pin contact
        let cc1 = fast_read_string(NODE_TRAN_CC1_STATUS, "");
        let cc2 = fast_read_string(NODE_TRAN_CC2_STATUS, "");
        let is_typec_fast = cc1.contains("3_0") || cc2.contains("3_0");

        // MediaTek Charger Type enum: 9 = FAST_CHARGER (PE / PE2.0 / PE5.0 / PD / Flash)
        let is_fast_charger_type = charger_type_num >= 9;

        // Algorithm path inspection (RFC / PumpExpress / PD handshake)
        let run_path = fast_read_string(NODE_TRAN_RUN_PATH, "");
        let is_pe_algo = run_path.contains("pe50") || run_path.contains("pe20") || run_path.contains("rfc_detected");

        let is_high_voltage = vbus_mv >= 8000; // 9V, 10V, 11V, 20V
        let is_flash_hw = primary_dvchg > 0
            || pump_express > 0
            || fast_indicator > 0
            || is_fast_charger_type
            || is_typec_fast
            || is_pe_algo
            || chg_mode.contains("fast")
            || chg_mode.contains("flash")
            || chg_type.contains("PE")
            || chg_type.contains("PD")
            || chg_type.contains("PPS")
            || chg_type.contains("FAST")
            || chg_type.contains("UFCS");

        let mut detected_w = 0;
        if adapter_cap > 0 {
            detected_w = adapter_cap;
        } else if ta_pwr >= 15 {
            detected_w = ta_pwr;
        } else if is_high_voltage && ibus_ma > 0 {
            detected_w = ((vbus_mv as i64 * ibus_ma as i64) / 1_000_000) as i32;
        }

        let is_pc_usb = is_sdp && !is_flash_hw && !is_high_voltage && detected_w < 15;
        st.is_pc_usb = is_pc_usb;

        if is_pc_usb {
            // Connected to PC via USB port (Standard Downstream Port)
            st.wattage = 5;
            st.is_supervooc = false;
            st.is_fast = false;
            st.tech = 0;
            st.fast_type = 0;
            st.ui_icon = 1;
            st.sup_speed = false;
        } else {
            // AC Wall Charger
            // Only trigger SuperVOOC if genuine fast charging is detected:
            // - detected wattage >= 15W (e.g. ta_power 33W or 70W)
            // - high voltage (>= 8V, e.g. PumpExpress 9V/11V, PD 9V/11V)
            // - flash hardware active (primary_dvchg > 0, Pump_Express > 0, fast_chg_indicator > 0)
            // - protocol is PE / PD / PPS / UFCS / FAST / FLASH
            // - user set persist.sys.charge_wattage >= 15
            let prop_wattage = Self::check_property_wattage();
            let is_fast_detected = detected_w >= 15
                || is_flash_hw
                || is_high_voltage
                || primary_dvchg > 0
                || prop_wattage.map(|w| w >= 15).unwrap_or(false);

            if is_fast_detected {
                let resolved_w = if let Some(pw) = prop_wattage {
                    pw
                } else if detected_w >= 15 {
                    detected_w
                } else {
                    33 // Baseline fast charge wattage on Camon 30
                };

                st.wattage = resolved_w;
                st.is_supervooc = true;
                st.is_fast = true;
                st.tech = 3; // SuperVOOC (required for ColorOS 17 animation & decimal trigger)
                st.fast_type = 2; // SuperVOOC 65W+
                st.ui_icon = 3; // ColorOS SuperVOOC animation
                st.sup_speed = true;
            } else {
                // Standard slow / normal AC wall charger (e.g. 5V 1A / 5V 2A 10W, normal powerbank)
                st.wattage = if detected_w > 0 { detected_w } else { 10 };
                st.is_supervooc = false;
                st.is_fast = false;
                st.tech = 0; // Normal AC
                st.fast_type = 0;
                st.ui_icon = 1; // Normal single bolt
                st.sup_speed = false;
            }
        }

        st.battery_level = fast_read_int(NODE_BATTERY_CAPACITY, 50);
        st.voltage_mv = Self::normalize_voltage(fast_read_int(NODE_BATTERY_VOLTAGE, 3800000));
        st.current_ma = Self::normalize_current(fast_read_int(NODE_BATTERY_CURRENT, 0));
        st.temp_tenths = fast_read_int(NODE_BATTERY_TEMP, 280);
        st.soh = fast_read_int(NODE_TRAN_HEALTH_PERCENT, 100).clamp(1, 100);
        st.cycle_count = Self::resolve_cycle_count();
        st.bypass_active = fast_read_int(NODE_TRAN_BYPASS_CHARGER, 0) != 0;
        st.limit_soc = fast_read_int(NODE_TRAN_SC_TUISOC, 100);
        st.charge_counter_mah = fast_read_int(NODE_BATTERY_CHARGE_COUNTER, 2500000) / 1000;
        st.charge_full_mah = fast_read_int(NODE_BATTERY_CHARGE_FULL, 5000000) / 1000;
        st.decimal_seed = read_int_any(UI_SOC_DECIMAL_PATHS);
        st.decimal_rate = read_int_any(UI_SOC_DECIMAL_RATE_PATHS);

        let rust = fast_read_int(NODE_TRAN_RUST_DETECT, 0);
        st.usb_status = if rust > 0 { 1 } else { 0 };

        st
    }

    pub fn set_bypass_charging(enabled: bool) -> bool {
        fast_write_int(NODE_TRAN_BYPASS_CHARGER, if enabled { 1 } else { 0 })
    }

    pub fn set_charge_limit(limit_percent: i32) -> bool {
        let val = if (1..=100).contains(&limit_percent) {
            limit_percent
        } else {
            100
        };
        fast_write_int(NODE_TRAN_SC_TUISOC, val)
    }

    pub fn next_decimal_soc(
        capacity: i32,
        is_charging: bool,
        wattage: i32,
        cached_seed: i32,
        cached_rate: i32,
    ) -> String {
        if !is_charging || capacity <= 0 {
            let mut state = DECIMAL_STATE.lock();
            state.capacity = -1;
            state.target = 0;
            return "0,0".to_string();
        }

        let capacity = capacity.clamp(0, 100);
        if capacity >= 100 {
            let mut state = DECIMAL_STATE.lock();
            state.capacity = 100;
            state.target = 10000;
            return "10000,10000".to_string();
        }

        let mut state = DECIMAL_STATE.lock();
        let (start, end) = synthetic_decimal_soc_pair(
            &mut state,
            capacity,
            cached_seed,
            cached_rate,
            wattage,
        );
        format!("{},{}", start, end)
    }

    pub fn build_update_json(&self) -> String {
        let (ui_icon, ui_power, fast_chg, sup_speed) = if self.online {
            (
                self.ui_icon,
                if self.is_fast { self.wattage } else { 0 },
                if self.is_fast { 1 } else { 0 },
                if self.sup_speed { 1 } else { 0 },
            )
        } else {
            (0, 0, 0, 0)
        };

        format!(
            "{{\"Update\":{{\"uiIconType\":{},\"uiPower\":{},\"cpaPower\":{},\"fastCharge\":{},\"isSupportSpeedCharge\":{},\"bypassChgStatus\":{},\"usbStatus\":{},\"otgOnline\":0,\"batteryRm\":{},\"batteryFcc\":{},\"shortIcOtpStatus\":0,\"shortCHwStatus\":0,\"notifyCode\":0,\"bmsStatus\":0,\"ppsCharging\":0,\"ppsChargePower\":0}}}}",
            ui_icon,
            ui_power,
            ui_power,
            fast_chg,
            sup_speed,
            if self.bypass_active { 1 } else { 0 },
            self.usb_status,
            self.charge_counter_mah,
            self.charge_full_mah,
        )
    }

    pub fn build_query_info(&self) -> String {
        format!(
            concat!(
                "bcc_exp_status=0\n",
                "battery_capacity={}\n",
                "battery_voltage_now={}\n",
                "battery_voltage_min=3000\n",
                "battery_temp={}\n",
                "battery_current_now={}\n",
                "battery_fcc={}\n",
                "battery_soh={}\n",
                "battery_cc={}\n",
                "battery_rm={}\n",
                "battery_status={}\n",
                "charge_technology={}\n",
                "fast_charge_type={}\n",
                "adapter_power_w={}\n",
                "is_fast_charge={}\n",
                "is_supervooc={}\n",
                "bypass_charger_status={}\n"
            ),
            self.battery_level,
            self.voltage_mv,
            self.temp_tenths,
            if self.online { self.current_ma.abs() } else { self.current_ma },
            self.charge_full_mah,
            self.soh,
            self.charge_counter_mah,
            self.charge_counter_mah,
            self.status,
            self.tech,
            self.fast_type,
            if self.is_fast { self.wattage } else { 0 },
            if self.is_fast { 1 } else { 0 },
            if self.is_supervooc { 1 } else { 0 },
            if self.bypass_active { 1 } else { 0 }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_percent_decimal_soc() {
        let mut state = DecimalSocState::default();
        let (start, end) = synthetic_decimal_soc_pair(&mut state, 0, 0, 0, 10);
        assert!((5..=50).contains(&start));
        assert!((5..=85).contains(&end));
    }

    #[test]
    fn test_synthetic_decimal_progresses_without_node() {
        let mut state = DecimalSocState::default();
        let (first_start, first_end) = synthetic_decimal_soc_pair(&mut state, 89, 0, 0, 10);
        let (second_start, second_end) = synthetic_decimal_soc_pair(&mut state, 89, 0, 0, 10);

        assert!((8905..=8950).contains(&first_start));
        assert!((8905..=8985).contains(&first_end));
        assert_ne!(first_end, first_start);
        assert_eq!(second_start, first_end);
        assert!((8905..=8985).contains(&second_end));
        assert_ne!(second_end, second_start);
    }

    #[test]
    fn test_synthetic_decimal_uses_node_seed_on_reset() {
        let mut state = DecimalSocState::default();
        let (first_start, first_end) = synthetic_decimal_soc_pair(&mut state, 89, 8942, 0, 10);
        let (second_start, second_end) = synthetic_decimal_soc_pair(&mut state, 89, 8901, 0, 10);

        assert_eq!(first_start, 8942);
        assert!((8905..=8985).contains(&first_end));
        assert_ne!(first_end, first_start);
        assert_eq!(second_start, first_end);
        assert!((8905..=8985).contains(&second_end));
        assert_ne!(second_end, second_start);
    }

    #[test]
    fn test_synthetic_decimal_does_not_regress_over_many_ticks() {
        let mut state = DecimalSocState::default();
        let mut previous_end = 0;

        for _ in 0..100 {
            let pair = synthetic_decimal_soc_pair(&mut state, 89, 8942, 0, 10);
            assert!((8905..=8999).contains(&pair.0));
            assert!((8905..=8999).contains(&pair.1));
            assert!(pair.1 >= pair.0);
            assert!(pair.1 >= previous_end);
            previous_end = pair.1;
        }
    }

    #[test]
    fn test_full_battery() {
        let mut state = DecimalSocState::default();
        let (start, end) = synthetic_decimal_soc_pair(&mut state, 100, 0, 0, 10);
        assert_eq!(start, 10000);
        assert_eq!(end, 10000);
    }

    #[test]
    fn test_offline_sentinel() {
        assert_eq!(TranssionState::next_decimal_soc(50, false, 33, 0, 0), "0,0");
        assert_eq!(TranssionState::next_decimal_soc(0, false, 0, 0, 0), "0,0");
    }

    #[test]
    fn test_rate_scaling_with_wattage() {
        assert_eq!(normalize_decimal_rate(0, 10), 1);
        assert_eq!(normalize_decimal_rate(0, 33), 2);
        assert_eq!(normalize_decimal_rate(0, 68), 3);
    }

    #[test]
    fn test_supervooc_update_json() {
        let mut st = TranssionState::default();
        st.online = true;
        st.is_fast = true;
        st.is_supervooc = true;
        st.tech = 3;
        st.fast_type = 2;
        st.ui_icon = 3;
        st.wattage = 33;
        st.sup_speed = true;
        st.usb_status = 0;
        st.charge_counter_mah = 2500;
        st.charge_full_mah = 5000;

        let json = st.build_update_json();
        assert!(json.contains("\"uiIconType\":3"));
        assert!(json.contains("\"uiPower\":33"));
        assert!(json.contains("\"cpaPower\":33"));
        assert!(json.contains("\"fastCharge\":1"));
        assert!(json.contains("\"isSupportSpeedCharge\":1"));
        assert!(json.contains("\"usbStatus\":0"));
    }

    #[test]
    fn test_pc_usb_update_json() {
        let mut st = TranssionState::default();
        st.online = true;
        st.is_pc_usb = true;
        st.is_fast = false;
        st.is_supervooc = false;
        st.tech = 0;
        st.fast_type = 0;
        st.ui_icon = 1;
        st.wattage = 5;
        st.sup_speed = false;
        st.usb_status = 0;

        let json = st.build_update_json();
        assert!(json.contains("\"uiIconType\":1"));
        assert!(json.contains("\"uiPower\":0"));
        assert!(json.contains("\"fastCharge\":0"));
        assert!(json.contains("\"isSupportSpeedCharge\":0"));
        assert!(json.contains("\"usbStatus\":0"));
    }

    #[test]
    fn test_supervooc_query_info() {
        let mut st = TranssionState::default();
        st.online = true;
        st.is_fast = true;
        st.is_supervooc = true;
        st.tech = 3;
        st.fast_type = 2;
        st.wattage = 33;

        let info = st.build_query_info();
        assert!(info.contains("charge_technology=3"));
        assert!(info.contains("is_supervooc=1"));
        assert!(info.contains("is_fast_charge=1"));
        assert!(info.contains("adapter_power_w=33"));
    }

    #[test]
    fn test_native_hardware_ui_soc_seed() {
        // Device sysfs reports 3705 for 37% battery
        let offset = normalize_decimal_seed_offset(3705, 37);
        assert_eq!(offset, Some(5));

        let mut state = DecimalSocState::default();
        let (start, end) = synthetic_decimal_soc_pair(&mut state, 37, 3705, 0, 33);
        assert_eq!(start, 3705);
        assert_eq!(end, 3707); // 33W rate step = 2
    }

    #[test]
    fn test_slow_ac_update_json() {
        let mut st = TranssionState::default();
        st.online = true;
        st.is_pc_usb = false;
        st.is_fast = false;
        st.is_supervooc = false;
        st.tech = 0;
        st.fast_type = 0;
        st.ui_icon = 1; // Slow 10W charger gets normal single bolt
        st.wattage = 10;
        st.sup_speed = false;
        st.usb_status = 0;

        let json = st.build_update_json();
        assert!(json.contains("\"uiIconType\":1"));
        assert!(json.contains("\"uiPower\":0"));
        assert!(json.contains("\"fastCharge\":0"));
        assert!(json.contains("\"isSupportSpeedCharge\":0"));
    }

    #[test]
    fn test_parse_toggle_flag() {
        assert!(parse_toggle_flag("1+switch=1"));
        assert!(parse_toggle_flag("switch=1"));
        assert!(parse_toggle_flag("1"));
        assert!(parse_toggle_flag("true"));
        assert!(parse_toggle_flag("enable"));
        assert!(parse_toggle_flag("on"));
        assert!(parse_toggle_flag("value=1+switch=1,reason=switch on"));

        // Crucial bugfix: 'reason' contains substring 'on', must NOT evaluate to true when switch=0!
        assert!(!parse_toggle_flag("1+switch=0"));
        assert!(!parse_toggle_flag("switch=0"));
        assert!(!parse_toggle_flag("0"));
        assert!(!parse_toggle_flag("false"));
        assert!(!parse_toggle_flag("disable"));
        assert!(!parse_toggle_flag("off"));
        assert!(!parse_toggle_flag("1+switch=0,reason=screen off"));
        assert!(!parse_toggle_flag("random_invalid_string"));
    }

    #[test]
    fn test_fast_read_int_parsing() {
        let dir = std::env::temp_dir();
        let path_dec = dir.join("test_fast_read_dec.txt");
        let path_hex = dir.join("test_fast_read_hex.txt");
        let path_txt = dir.join("test_fast_read_txt.txt");
        let path_neg = dir.join("test_fast_read_neg.txt");

        std::fs::write(&path_dec, "33\n").unwrap();
        std::fs::write(&path_hex, "0x1f\n").unwrap();
        std::fs::write(&path_txt, "9 (FAST_CHARGER)\n").unwrap();
        std::fs::write(&path_neg, "-6100\n").unwrap();

        assert_eq!(fast_read_int(path_dec.to_str().unwrap(), 0), 33);
        assert_eq!(fast_read_int(path_hex.to_str().unwrap(), 0), 31);
        assert_eq!(fast_read_int(path_txt.to_str().unwrap(), 0), 9);
        assert_eq!(fast_read_int(path_neg.to_str().unwrap(), 0), -6100);
        assert_eq!(fast_read_int("/non/existent/path/test.txt", -99), -99);

        let _ = std::fs::remove_file(path_dec);
        let _ = std::fs::remove_file(path_hex);
        let _ = std::fs::remove_file(path_txt);
        let _ = std::fs::remove_file(path_neg);
    }
}

