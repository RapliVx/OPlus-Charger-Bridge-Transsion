/*
 * Copyright (C) 2026 Project EVA / CosFix
 *
 * Transsion Hardware Subsystem Bridge for MediaTek & Transsion Platforms
 * (Tecno Camon 30 / Helio G99 / MT6789 / Dimensity / Infinix / Itel)
 */

use std::sync::atomic::{AtomicU32, Ordering};

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

static DECIMAL_COUNTER: AtomicU32 = AtomicU32::new(10);

pub fn fast_read_int(path: &str, default_val: i32) -> i32 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
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
}

impl TranssionState {
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
        let is_sdp = usb_type_str.contains("[SDP]") || charger_type_num == 0;

        // Transsion Hardware Sysfs Inspection
        let adapter_cap = fast_read_int(NODE_TRAN_ADAPTER_CAPACITY, 0);
        let ta_pwr = fast_read_int(NODE_TRAN_TA_POWER, 0);
        let pump_express = fast_read_int(NODE_TRAN_PUMP_EXPRESS, 0);
        let primary_dvchg = fast_read_int(NODE_TRAN_PRIMARY_DVCHG, 0);
        let fast_indicator = fast_read_int(NODE_TRAN_FAST_CHG, 0);
        let vbus_mv = fast_read_int(NODE_TRAN_ADC_VBUS, 0);
        let ibus_ma = fast_read_int(NODE_TRAN_ADC_IBUS, 0);

        let chg_mode = fast_read_string(NODE_TRAN_CHARGING_MODE, "");
        let chg_type = fast_read_string(NODE_TRAN_CHG_TYPE, "");

        let is_high_voltage = vbus_mv >= 8000; // 9V, 10V, 11V, 20V
        let is_flash_hw = primary_dvchg > 0
            || pump_express > 0
            || fast_indicator > 0
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
        } else if ta_pwr > 0 {
            detected_w = ta_pwr;
        } else if is_high_voltage && ibus_ma > 0 {
            detected_w = ((vbus_mv as i64 * ibus_ma as i64) / 1_000_000) as i32;
        }

        let is_pc_usb = is_sdp && !is_flash_hw && !is_high_voltage && detected_w < 15;
        st.is_pc_usb = is_pc_usb;

        // Check user / runtime property override
        let prop_wattage = Self::check_property_wattage();
        if let Some(prop_w) = prop_wattage {
            st.wattage = prop_w;
            if prop_w >= 30 {
                st.is_supervooc = true;
                st.is_fast = true;
                st.tech = 3; // SuperVOOC
                st.fast_type = 2; // SuperVOOC 65W+
                st.ui_icon = 3; // ColorOS SuperVOOC animation
                st.sup_speed = true;
            } else if prop_w >= 15 {
                st.is_supervooc = false;
                st.is_fast = true;
                st.tech = 2; // VOOC / Fast
                st.fast_type = 1;
                st.ui_icon = 2; // ColorOS VOOC animation
                st.sup_speed = false;
            } else {
                st.is_supervooc = false;
                st.is_fast = false;
                st.tech = 0;
                st.fast_type = 0;
                st.ui_icon = 1;
                st.sup_speed = false;
            }
        } else if is_pc_usb {
            // Connected to PC via USB port (Standard Downstream Port)
            st.wattage = 5;
            st.is_supervooc = false;
            st.is_fast = false;
            st.tech = 0;
            st.fast_type = 0;
            st.ui_icon = 1;
            st.sup_speed = false;
        } else {
            // Classification of charging protocols:
            // SuperVOOC: Wattage >= 25W OR charge pump active (primary_dvchg > 0)
            if detected_w >= 25 || (primary_dvchg > 0 && detected_w >= 18) {
                st.wattage = if detected_w > 0 { detected_w } else { 33 };
                st.is_supervooc = true;
                st.is_fast = true;
                st.tech = 3; // SuperVOOC
                st.fast_type = 2; // SuperVOOC 65W+
                st.ui_icon = 3; // ColorOS SuperVOOC animation
                st.sup_speed = true;
            } else if detected_w >= 15 || is_flash_hw {
                st.wattage = if detected_w > 0 { detected_w } else { 18 };
                st.is_supervooc = false;
                st.is_fast = true;
                st.tech = 2; // VOOC / Fast
                st.fast_type = 1;
                st.ui_icon = 2; // ColorOS VOOC animation
                st.sup_speed = false;
            } else {
                st.wattage = if detected_w > 0 { detected_w } else { 10 };
                st.is_supervooc = false;
                st.is_fast = false;
                st.tech = 0; // Normal AC
                st.fast_type = 0;
                st.ui_icon = 1; // Normal bolt
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

    pub fn next_decimal_soc(capacity: i32, is_charging: bool) -> String {
        if !is_charging {
            return String::new();
        }
        let counter = DECIMAL_COUNTER.fetch_add(7, Ordering::Relaxed) % 100;
        format!("{}.{:02}", capacity, counter)
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

        // In ColorOS, usbStatus is a bitmask of USB port hardware abnormality/error:
        // Bit 0 (0x1) = HW abnormal / debris / short detected (triggers notification 39)
        // Bit 5 (0x20) = liquid detected
        // 0 = Normal / clean / dry (expected for all normal charging and PC USB)
        let rust = fast_read_int(NODE_TRAN_RUST_DETECT, 0);
        let usb_status = if rust > 0 { 1 } else { 0 };

        format!(
            "{{\"Update\":{{\"uiIconType\":{},\"uiPower\":{},\"cpaPower\":{},\"fastCharge\":{},\"isSupportSpeedCharge\":{},\"bypassChgStatus\":{},\"usbStatus\":{},\"otgOnline\":0,\"batteryRm\":{},\"batteryFcc\":{},\"shortIcOtpStatus\":0,\"shortCHwStatus\":0,\"notifyCode\":0,\"bmsStatus\":0,\"ppsCharging\":0,\"ppsChargePower\":0}}}}",
            ui_icon,
            ui_power,
            ui_power,
            fast_chg,
            sup_speed,
            if self.bypass_active { 1 } else { 0 },
            usb_status,
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
            self.current_ma,
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

