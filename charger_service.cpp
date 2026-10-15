/*
 * Copyright (C) 2026 Project EVA / CosFix
 *
 * Transsion Family Charger AIDL HAL Adapter for ColorOS / OxygenOS / RealmeUI ROMs
 * Target: Tecno / Infinix / Itel (MediaTek MT6789 / Helio G99 / Dimensity / Transsion Platforms)
 *
 * Implements: vendor.oplus.hardware.charger.ICharger/default
 * Version: 1
 * Hash: 049782b2370590b7ce37aec3e3070adcd99f96fe
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <unistd.h>
#include <fcntl.h>
#include <inttypes.h>
#include <signal.h>
#include <sys/system_properties.h>

#include <android/log.h>
#include <android/binder_ibinder.h>
#include <android/binder_status.h>
#include <android/binder_parcel.h>

#ifdef __cplusplus
extern "C" {
#endif

/* From <android/binder_manager.h> */
binder_status_t AServiceManager_addService(AIBinder* binder, const char* instance);
AIBinder* AServiceManager_checkService(const char* instance);
AIBinder* AServiceManager_waitForService(const char* instance);
bool AServiceManager_isDeclared(const char* instance);

/* From <android/binder_ibinder.h> */
void AIBinder_markVintfStability(AIBinder* binder);

/* From <android/binder_process.h> */
void ABinderProcess_startThreadPool(void);
void ABinderProcess_joinThreadPool(void);
bool ABinderProcess_setThreadPoolMaxThreadCount(uint32_t numThreads);

#ifdef __cplusplus
}
#endif

#define LOG_TAG "tran_charger_hal"
#define ALOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define ALOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define ALOGW(...) __android_log_print(ANDROID_LOG_WARN, LOG_TAG, __VA_ARGS__)
#define ALOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

/* Hot-path debug logging: throttled unless persist.sys.charger.debug or debug.charger is enabled */
static inline bool is_debug_logging_enabled(void) {
    char val[PROP_VALUE_MAX] = {0};
    if (__system_property_get("persist.sys.charger.debug", val) > 0 && val[0] == '1') {
        return true;
    }
    if (__system_property_get("debug.charger", val) > 0 && val[0] == '1') {
        return true;
    }
    return false;
}
#define ALOGD_CHARGER(...) do { if (__builtin_expect(is_debug_logging_enabled(), 0)) { ALOGD(__VA_ARGS__); } } while (0)

#define CHARGER_INTERFACE_DESCRIPTOR "vendor.oplus.hardware.charger.ICharger"
#define CHARGER_INSTANCE_NAME        "vendor.oplus.hardware.charger.ICharger/default"
#define CHARGER_INTERFACE_HASH       "049782b2370590b7ce37aec3e3070adcd99f96fe"
#define CHARGER_INTERFACE_VERSION    1

/* Standard Sysfs Nodes */
#define NODE_BATTERY_CAPACITY        "/sys/class/power_supply/battery/capacity"
#define NODE_BATTERY_VOLTAGE         "/sys/class/power_supply/battery/voltage_now"
#define NODE_BATTERY_CURRENT         "/sys/class/power_supply/battery/current_now"
#define NODE_BATTERY_TEMP            "/sys/class/power_supply/battery/temp"
#define NODE_BATTERY_STATUS          "/sys/class/power_supply/battery/status"
#define NODE_BATTERY_CHARGE_FULL     "/sys/class/power_supply/battery/charge_full"
#define NODE_BATTERY_CHARGE_COUNTER  "/sys/class/power_supply/battery/charge_counter"
#define NODE_BATTERY_CYCLE_COUNT     "/sys/class/power_supply/battery/cycle_count"

#define NODE_CHARGER_ONLINE          "/sys/class/power_supply/charger/online"
#define NODE_CHARGER_TYPE            "/sys/class/power_supply/charger/type"
#define NODE_CHARGER_INPUT_LIMIT     "/sys/class/power_supply/charger/input_current_limit"
#define NODE_CHARGER_USB_TYPE        "/sys/class/power_supply/charger/usb_type"

/* Transsion & MediaTek Hardware Fast Charge Nodes */
#define NODE_TRAN_ADAPTER_CAPACITY   "/sys/devices/platform/charger/adapter_capacity"
#define NODE_TRAN_TA_POWER           "/sys/class/power_supply/tran-auto-test/ta_power"
#define NODE_TRAN_CHARGING_MODE      "/sys/devices/platform/charger/Charging_mode"
#define NODE_TRAN_CHARGER_TYPE       "/sys/devices/platform/charger/Charger_Type"
#define NODE_TRAN_PD_TYPE            "/sys/devices/platform/charger/pd_type"
#define NODE_TRAN_PUMP_EXPRESS       "/sys/devices/platform/charger/Pump_Express"
#define NODE_TRAN_FAST_CHG           "/sys/devices/platform/charger/fast_chg_indicator"
#define NODE_TRAN_ADC_VBUS           "/sys/devices/platform/charger/ADC_Charger_Voltage"
#define NODE_TRAN_ADC_IBUS           "/sys/devices/platform/charger/ADC_Charging_Current"
#define NODE_TRAN_PRIMARY_DVCHG      "/sys/class/power_supply/primary_dvchg/present"
#define NODE_TRAN_CHG_TYPE           "/sys/class/power_supply/tran-auto-test/chg_type"
#define NODE_TRAN_CHG_STATUS         "/sys/class/power_supply/tran-auto-test/chg_status"
#define NODE_TRAN_UI_SOC             "/sys/class/power_supply/tran-auto-test/ui_soc"

/* Transsion Feature Controls */
#define NODE_TRAN_BYPASS_CHARGER     "/sys/devices/platform/charger/bypass_charger"
#define NODE_TRAN_HEALTH_PERCENT     "/sys/devices/platform/charger/battery_health_percent"
#define NODE_TRAN_BATTERY_CYCLE      "/sys/devices/platform/charger/tran_battery_cycle"
#define NODE_TRAN_SC_TUISOC          "/sys/devices/platform/charger/sc_tuisoc"

/* Global immutable OK status header (allocated once in main, zero heap churn across Binder calls) */
static AStatus *g_ok_status = NULL;

/* Decimal battery SOC simulation counter for fallback decimal animation */
static uint32_t s_decimal_counter = 0;

/* High-performance POSIX file helpers (zero stdio heap buffer allocation) */
static int fast_read_int(const char *path, int default_val) {
    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0) return default_val;
    char buf[32];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) return default_val;
    buf[n] = '\0';
    return atoi(buf);
}

static void fast_read_string(const char *path, char *buf, size_t max_len, const char *default_val) {
    int fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0) {
        strncpy(buf, default_val, max_len - 1);
        buf[max_len - 1] = '\0';
        return;
    }
    ssize_t n = read(fd, buf, max_len - 1);
    close(fd);
    if (n <= 0) {
        strncpy(buf, default_val, max_len - 1);
        buf[max_len - 1] = '\0';
        return;
    }
    buf[n] = '\0';
    size_t len = (size_t)n;
    while (len > 0 && (buf[len - 1] == '\n' || buf[len - 1] == '\r' || buf[len - 1] == ' ')) {
        buf[len - 1] = '\0';
        len--;
    }
}

__attribute__((unused))
static bool fast_write_string(const char *path, const char *val) {
    int fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) {
        ALOGW("Failed to open %s for write", path);
        return false;
    }
    size_t len = strlen(val);
    ssize_t w = write(fd, val, len);
    close(fd);
    return (w == (ssize_t)len);
}

static bool fast_write_int(const char *path, int val) {
    char buf[32];
    int len = snprintf(buf, sizeof(buf), "%d", val);
    int fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) {
        ALOGW("Failed to open %s for write", path);
        return false;
    }
    ssize_t w = write(fd, buf, (size_t)len);
    close(fd);
    return (w == (ssize_t)len);
}

/* Transsion Telemetry Helpers */
static inline int get_battery_level(void) {
    return fast_read_int(NODE_BATTERY_CAPACITY, 50);
}

static inline int get_battery_voltage_mv(void) {
    int v = fast_read_int(NODE_BATTERY_VOLTAGE, 3800000);
    return (v > 100000) ? (v / 1000) : v;
}

static inline int get_battery_current_ma(void) {
    int cur = fast_read_int(NODE_BATTERY_CURRENT, 0);
    return (abs(cur) > 10000) ? (cur / 1000) : cur;
}

static inline int get_battery_temp_tenths(void) {
    return fast_read_int(NODE_BATTERY_TEMP, 280);
}

static inline bool is_usb_online(void) {
    if (fast_read_int(NODE_CHARGER_ONLINE, 0) != 0) return true;
    char stat[32] = {0};
    fast_read_string(NODE_BATTERY_STATUS, stat, sizeof(stat), "");
    if (strcmp(stat, "Charging") == 0 || strcmp(stat, "Full") == 0) return true;
    return false;
}

static inline int get_battery_soh(void) {
    int soh = fast_read_int(NODE_TRAN_HEALTH_PERCENT, 100);
    return (soh > 0 && soh <= 100) ? soh : 100;
}

static inline int get_cycle_count(void) {
    int cyc = fast_read_int(NODE_TRAN_BATTERY_CYCLE, -1);
    if (cyc >= 0) return cyc;
    return fast_read_int(NODE_BATTERY_CYCLE_COUNT, 0);
}

/*
 * Accurate Charger State Structure
 * Differentiates Disconnected vs Normal (5W/10W) vs Fast Charge (18W) vs SuperVOOC (33W+)
 */
struct ChargerState {
    bool online;
    int wattage;        /* in Watts */
    bool is_fast;       /* Fast charging active */
    bool is_supervooc;  /* Real SuperVOOC active */
    int tech;           /* 0 = Normal, 1 = QC/PD, 2 = VOOC, 3 = SuperVOOC */
    int fast_type;      /* 0 = Normal, 1 = VOOC, 2 = SuperVOOC 65W+ */
    int ui_icon;        /* 0 = none, 1 = normal/fast bolt, 2 = SuperVOOC animation */
    bool sup_speed;     /* 1 if SuperVOOC, 0 otherwise */
};

static inline struct ChargerState resolve_charger_state(void) {
    struct ChargerState state;
    memset(&state, 0, sizeof(state));

    state.online = is_usb_online();
    if (!state.online) {
        return state; /* All 0 when unplugged */
    }

    /* 1. Check user / runtime property override */
    char prop_val[PROP_VALUE_MAX] = {0};
    int prop_w = 0;
    if (__system_property_get("persist.sys.charge_wattage", prop_val) > 0 && prop_val[0] != '\0') {
        prop_w = atoi(prop_val);
    } else if (__system_property_get("persist.sys.oplus.charge_wattage", prop_val) > 0 && prop_val[0] != '\0') {
        prop_w = atoi(prop_val);
    } else if (__system_property_get("persist.sys.rianixia.charge-wattage", prop_val) > 0 && prop_val[0] != '\0') {
        prop_w = atoi(prop_val);
    }

    if (prop_w > 0) {
        state.wattage = prop_w;
        if (prop_w >= 30) {
            state.is_supervooc = true;
            state.is_fast = true;
            state.tech = 3;       /* SuperVOOC */
            state.fast_type = 2;  /* SuperVOOC 65W+ */
            state.ui_icon = 2;    /* SuperVOOC logo & glowing ring */
            state.sup_speed = true;
        } else if (prop_w >= 15) {
            state.is_supervooc = false;
            state.is_fast = true;
            state.tech = 2;       /* VOOC / Fast charge */
            state.fast_type = 1;  /* VOOC */
            state.ui_icon = 1;    /* Fast charge bolt */
            state.sup_speed = false;
        } else {
            /* Low wattage override -> Normal charging */
            state.is_supervooc = false;
            state.is_fast = false;
            state.tech = 0;
            state.fast_type = 0;
            state.ui_icon = 1;    /* Standard charging bolt */
            state.sup_speed = false;
        }
        return state;
    }

    /* 2. Read Transsion Hardware Adapter Capacity & Travel Adapter Power */
    int adapter_cap = fast_read_int(NODE_TRAN_ADAPTER_CAPACITY, 0);
    int ta_pwr = fast_read_int(NODE_TRAN_TA_POWER, 0);

    /* 3. Read Hardware Fast Charging & Protocol Indicators */
    char chg_mode[32] = {0};
    fast_read_string(NODE_TRAN_CHARGING_MODE, chg_mode, sizeof(chg_mode), "");

    char chg_type[32] = {0};
    fast_read_string(NODE_TRAN_CHG_TYPE, chg_type, sizeof(chg_type), "");

    int pump_express = fast_read_int(NODE_TRAN_PUMP_EXPRESS, 0);
    int primary_dvchg = fast_read_int(NODE_TRAN_PRIMARY_DVCHG, 0);
    int vbus_mv = fast_read_int(NODE_TRAN_ADC_VBUS, 0);
    int ibus_ma = fast_read_int(NODE_TRAN_ADC_IBUS, 0);

    /* Fast Charging Flag detection from hardware nodes */
    bool is_high_voltage = (vbus_mv >= 8000); /* 8V+, e.g. 9V/10V/11V/20V */
    bool is_flash_hw = (primary_dvchg > 0) || (pump_express > 0) ||
                       (strstr(chg_mode, "fast") != NULL) || (strstr(chg_mode, "flash") != NULL) ||
                       (strstr(chg_type, "PE") != NULL) || (strstr(chg_type, "PD") != NULL) ||
                       (strstr(chg_type, "PPS") != NULL) || (strstr(chg_type, "FAST") != NULL) ||
                       (strstr(chg_type, "UFCS") != NULL);

    /* 4. Determine Actual Wattage */
    int detected_w = 0;
    if (adapter_cap > 0) {
        detected_w = adapter_cap;
    } else if (ta_pwr > 0) {
        detected_w = ta_pwr;
    } else if (is_high_voltage && ibus_ma > 0) {
        detected_w = (int)(((long long)vbus_mv * ibus_ma) / 1000000LL);
    }

    /* 5. Classification of Real Charging State */
    /*
     * SuperVOOC:
     * - Hardware negotiated wattage >= 30W (e.g. 33W, 45W, 68W, 70W, 120W)
     * - OR direct charge pump active (primary_dvchg > 0)
     * - OR high-voltage flash mode with capacity >= 30W
     */
    if (detected_w >= 30 || (primary_dvchg > 0 && detected_w >= 20)) {
        state.wattage = (detected_w > 0) ? detected_w : 33;
        state.is_supervooc = true;
        state.is_fast = true;
        state.tech = 3;       /* SuperVOOC */
        state.fast_type = 2;  /* SuperVOOC 65W+ */
        state.ui_icon = 2;    /* SuperVOOC UI animation */
        state.sup_speed = true;
    }
    /*
     * Fast Charging (15W - 29W, e.g. 18W QC/PD/PE):
     */
    else if (detected_w >= 15 || (is_flash_hw && !is_high_voltage)) {
        state.wattage = (detected_w > 0) ? detected_w : 18;
        state.is_supervooc = false;
        state.is_fast = true;
        state.tech = 2;       /* VOOC / Fast */
        state.fast_type = 1;  /* VOOC */
        state.ui_icon = 1;    /* Standard fast charge bolt */
        state.sup_speed = false;
    }
    /*
     * Normal / Standard Charging (SDP, CDP, DCP 5W - 10W):
     * Real PC USB / 5V chargers. SuperVOOC is NOT triggered!
     */
    else {
        state.wattage = (detected_w > 0) ? detected_w : 10;
        state.is_supervooc = false;
        state.is_fast = false;
        state.tech = 0;       /* Normal */
        state.fast_type = 0;  /* Normal */
        state.ui_icon = 1;    /* Standard green charging bolt */
        state.sup_speed = false;
    }

    return state;
}

/* String allocator for AParcel_readString */
static bool parcel_string_allocator(void *stringData, int32_t length, char **buffer) {
    if (length < 0) return false;
    char *str = (char *)malloc(length + 1);
    if (!str) return false;
    str[length] = '\0';
    *(char **)stringData = str;
    *buffer = str;
    return true;
}

/*
 * ICharger Binder onTransact handler (130 methods + version/hash)
 */
static binder_status_t charger_onTransact(AIBinder *binder, transaction_code_t code,
                                          const AParcel *in, AParcel *out) {
    (void)binder;

    /* Always start reply with pre-allocated immutable OK status header (zero heap churn) */
    AParcel_writeStatusHeader(out, g_ok_status);

    struct ChargerState st = resolve_charger_state();

    switch (code) {
        /*
         * Active Telemetry & Metric Getters
         */
        case 4: /* getAcType */
            AParcel_writeInt32(out, st.online ? 1 : 0);
            return STATUS_OK;

        case 5:  /* getBattAuthenticate */
        case 30: /* getPsyBatteryHmac */
            AParcel_writeInt32(out, 1); /* 1 = Genuine Battery */
            return STATUS_OK;

        case 6:  /* getBattPPSChgIng */
            AParcel_writeInt32(out, st.is_fast ? 1 : 0);
            return STATUS_OK;

        case 7:  /* getBattPPSChgPower */
            AParcel_writeInt32(out, st.is_fast ? st.wattage : 0);
            return STATUS_OK;

        case 11: /* getBattVoocChgIng - REAL SuperVOOC trigger only */
            AParcel_writeInt32(out, st.is_supervooc ? 1 : 0);
            return STATUS_OK;

        case 12: /* getBatteryVoltageNow */
            AParcel_writeInt32(out, get_battery_voltage_mv());
            return STATUS_OK;

        case 24: /* getFastCharge - REAL Fast charge active trigger */
            AParcel_writeInt32(out, st.is_fast ? 1 : 0);
            return STATUS_OK;

        case 26: /* getPsyAcOnline */
            AParcel_writeInt32(out, st.online ? 1 : 0);
            return STATUS_OK;

        case 27: /* getPsyBatteryCC */
            AParcel_writeInt32(out, fast_read_int(NODE_BATTERY_CHARGE_COUNTER, 2500000) / 1000);
            return STATUS_OK;

        case 28: /* getPsyBatteryCurrentNow */
            AParcel_writeInt32(out, get_battery_current_ma());
            return STATUS_OK;

        case 29: /* getPsyBatteryFcc */
            AParcel_writeInt32(out, fast_read_int(NODE_BATTERY_CHARGE_FULL, 5000000) / 1000);
            return STATUS_OK;

        case 31: /* getPsyBatteryLevel */
        case 64: /* getWirelessCapacity */
            AParcel_writeInt32(out, get_battery_level());
            return STATUS_OK;

        case 35: /* getPsyBatteryRm */
            AParcel_writeInt32(out, fast_read_int(NODE_BATTERY_CHARGE_COUNTER, 2500000) / 1000);
            return STATUS_OK;

        case 38: { /* getPsyBatteryStatus */
            char stat[32];
            fast_read_string(NODE_BATTERY_STATUS, stat, sizeof(stat), st.online ? "Charging" : "Discharging");
            AParcel_writeString(out, stat, (int32_t)strlen(stat));
            return STATUS_OK;
        }

        case 39: /* getPsyBatteryTemp */
            AParcel_writeInt32(out, get_battery_temp_tenths());
            return STATUS_OK;

        case 40: /* getPsyChargeTech: 3 = SuperVOOC, 2 = VOOC, 1 = QC/PD, 0 = Normal */
            AParcel_writeInt32(out, st.tech);
            return STATUS_OK;

        case 41: /* getPsyFastChgType: 2 = SuperVOOC 65W+, 1 = VOOC, 0 = Normal */
            AParcel_writeInt32(out, st.fast_type);
            return STATUS_OK;

        case 42: /* getPsyInputCurrent */
        case 60: /* getUsbInputCurrentNow */
            AParcel_writeInt32(out, abs(get_battery_current_ma()));
            return STATUS_OK;

        case 48: /* getPsyUsbOnline */
            AParcel_writeInt32(out, st.online ? 1 : 0);
            return STATUS_OK;

        case 58: /* getUIsohValue */
            AParcel_writeInt32(out, get_battery_soh());
            return STATUS_OK;

        case 63: /* getWirelessAdapterPower */
            AParcel_writeInt32(out, 0);
            return STATUS_OK;

        case 76: { /* healthd_update_ui_soc_decimal - Real decimal battery percent animation */
            char dec_soc[32];
            int hw_ui_soc = fast_read_int(NODE_TRAN_UI_SOC, -1);
            if (hw_ui_soc > 0) {
                /* Hardware real fuel gauge decimal SOC (e.g. 2955 -> 29.55%) */
                int int_part = hw_ui_soc / 100;
                int frac_part = hw_ui_soc % 100;
                snprintf(dec_soc, sizeof(dec_soc), "%d.%02d", int_part, frac_part);
            } else {
                int level = get_battery_level();
                if (st.is_supervooc) {
                    s_decimal_counter = (s_decimal_counter + 7) % 100;
                    snprintf(dec_soc, sizeof(dec_soc), "%d.%02u", level, s_decimal_counter);
                } else {
                    snprintf(dec_soc, sizeof(dec_soc), "%d.0", level);
                }
            }
            AParcel_writeString(out, dec_soc, (int32_t)strlen(dec_soc));
            return STATUS_OK;
        }

        case 79: { /* queryChargeInfo - ColorOS standard key=value contract */
            char info_buf[2048];
            int level = get_battery_level();
            int vbat = get_battery_voltage_mv();
            int ibat = get_battery_current_ma();
            int temp = get_battery_temp_tenths();
            int cc = fast_read_int(NODE_BATTERY_CHARGE_COUNTER, 2500000) / 1000;
            int soh = get_battery_soh();

            snprintf(info_buf, sizeof(info_buf),
                "bcc_exp_status=0\n"
                "battery_capacity=%d\n"
                "battery_voltage_now=%d\n"
                "battery_voltage_min=0\n"
                "battery_temp=%d\n"
                "battery_current_now=%d\n"
                "battery_charge_now=%d\n"
                "battery_sub_current=0\n"
                "usb_fast_chg_type=%d\n"
                "battery_voocchg_ing=%d\n"
                "battery_ppschg_ing=%d\n"
                "battery_ppschg_power=%d\n"
                "usb_input_current_now=%d\n"
                "battery_short_ic_otp_status=0\n"
                "battery_authenticate=1\n"
                "battery_bqfs_status=0\n"
                "gauge_ibat_deviation=0\n"
                "battery_charge_technology=%d\n"
                "parallel_chg_mos_status=0\n"
                "wireless_current_now=0\n"
                "wireless_rx_version=\n"
                "wireless_tx_version=\n"
                "wireless_idt_adc_test=0\n"
                "wireless_enable_tx=4\n"
                "battery_status=%d\n"
                "wireless_voltage_now=0\n"
                "wireless_real_type=\n"
                "wireless_charger_type=0\n"
                "wireless_charge_pump_en=0\n"
                "wireless_deviated=\n"
                "battery_temp_not_plug=0\n"
                "battery_voltage_max_not_plug=0\n"
                "battery_voltage_min_not_plug=0\n"
                "dual_chan_support=0\n"
                "dual_chan_vbat_status=0\n"
                "dual_chan_buck_status=0\n"
                "dual_chan_temp_range_status=0\n"
                "chargerAcOnline=%d\n"
                "\n"
                "bob_status=0\n"
                "bob_status_reg=0\n"
                "ttf_info=0,0\n"
                "bsl_data=\n"
                "eis_data=\n"
                "battery_type_str=Li-ion\n"
                "batt_chemID=0\n"
                "battery_uisoh=%d\n"
                "battery_uisoh_is_100=%d\n"
                "battery_realsoh=%d\n",
                level, vbat, temp, ibat, cc,
                st.fast_type,
                st.is_supervooc ? 1 : 0,
                st.is_fast ? 1 : 0,
                st.is_fast ? st.wattage : 0,
                abs(ibat),
                st.tech,
                st.online ? 1 : 2,
                st.online ? 1 : 0,
                soh, (soh == 100) ? 1 : 0, soh
            );
            AParcel_writeString(out, info_buf, (int32_t)strlen(info_buf));
            return STATUS_OK;
        }

        case 125: /* getPsyBatterySN */
            AParcel_writeString(out, "TRAN-BATT-001", 13);
            return STATUS_OK;

        case 126: /* getBattGaugeInfo */
            AParcel_writeString(out, "MT6358-GAUGE", 12);
            return STATUS_OK;

        /*
         * Setters & Config Handlers
         */
        case 78:
        case 90: { /* notifyScreenStatus */
            int32_t screen_status = 0;
            if (in) AParcel_readInt32(in, &screen_status);
            ALOGD_CHARGER("notifyScreenStatus: %d", screen_status);
            AParcel_writeInt32(out, 0);
            return STATUS_OK;
        }

        case 127: { /* setChgConfig */
            int32_t flag = 0;
            int32_t caller_name = 0;
            char *extra_str = NULL;

            if (in) {
                AParcel_readInt32(in, &flag);
                AParcel_readString(in, &extra_str, parcel_string_allocator);
                AParcel_readInt32(in, &caller_name);
                (void)caller_name;
            }

            ALOGI("setChgConfig flag: %d, payload: %s", flag, extra_str ? extra_str : "null");

            if (flag == 24 || flag == 25) {
                /* Bypass Charging toggle / set */
                bool enable = false;
                if (extra_str && extra_str[0] != '\0') {
                    if (strstr(extra_str, "switch=1") || strcmp(extra_str, "1") == 0 ||
                        strstr(extra_str, "true") || strstr(extra_str, "enable") || strstr(extra_str, "on")) {
                        enable = true;
                    }
                } else {
                    enable = (fast_read_int(NODE_TRAN_BYPASS_CHARGER, 0) == 0);
                }
                fast_write_int(NODE_TRAN_BYPASS_CHARGER, enable ? 1 : 0);
                ALOGI("Bypass charging hardware node updated: %d", enable ? 1 : 0);
            } else if (flag == 14) {
                /* Charge Limit switch (Stop charging at 80% / battery protect) */
                bool limit_on = false;
                if (extra_str && extra_str[0] != '\0') {
                    if (strstr(extra_str, "switch=1") || strcmp(extra_str, "1") == 0 ||
                        strstr(extra_str, "true") || strstr(extra_str, "enable") || strstr(extra_str, "on")) {
                        limit_on = true;
                    }
                }
                fast_write_int(NODE_TRAN_SC_TUISOC, limit_on ? 80 : 100);
                ALOGI("Charge limit hardware node updated: %d", limit_on ? 80 : 100);
            } else if (flag == 5) {
                /* Target SOC Limit */
                int target = 80;
                if (extra_str && extra_str[0] != '\0') {
                    int p = atoi(extra_str);
                    if (p > 0 && p <= 100) target = p;
                }
                fast_write_int(NODE_TRAN_SC_TUISOC, target);
                ALOGI("Charge target SOC hardware node updated: %d%%", target);
            }

            if (extra_str) free(extra_str);
            AParcel_writeInt32(out, 0); /* 0 = Success */
            return STATUS_OK;
        }

        case 128: { /* getChgConfig - Called directly by OplusBatteryService */
            int32_t flag = 0;
            if (in) AParcel_readInt32(in, &flag);

            char res_buf[256];
            res_buf[0] = '\0';

            switch (flag) {
                case 0: /* mChargerTechnology: 3 = SuperVOOC, 2 = VOOC, 0 = Normal */
                    snprintf(res_buf, sizeof(res_buf), "%d", st.tech);
                    break;
                case 1: /* mChargeWattage */
                case 2:
                case 3:
                case 4: /* mCPAChargeWattage */
                    /* Returns real wattage if fast/supervooc, or 0 / low wattage for normal */
                    snprintf(res_buf, sizeof(res_buf), "%d", st.is_fast ? st.wattage : 0);
                    break;
                case 5: /* Charge limit percent */
                    snprintf(res_buf, sizeof(res_buf), "%d", fast_read_int(NODE_TRAN_SC_TUISOC, 80));
                    break;
                case 6: /* mIsSupSpeedCharge: 1 = true only if REAL SuperVOOC, 0 = false */
                    snprintf(res_buf, sizeof(res_buf), "%d", st.sup_speed ? 1 : 0);
                    break;
                case 14: /* Limit enabled */
                    snprintf(res_buf, sizeof(res_buf), "%d", (fast_read_int(NODE_TRAN_SC_TUISOC, 100) < 100) ? 1 : 0);
                    break;
                case 20: /* Battery cycles */
                    snprintf(res_buf, sizeof(res_buf), "%d", get_cycle_count());
                    break;
                case 21: /* Silicon battery */
                    strcpy(res_buf, "0");
                    break;
                case 23: /* Gauge type */
                    strcpy(res_buf, "mtk");
                    break;
                case 24:
                case 25: /* Bypass charging status */
                    snprintf(res_buf, sizeof(res_buf), "%d", fast_read_int(NODE_TRAN_BYPASS_CHARGER, 0));
                    break;
                case 30: /* getDataFromHalByUpdate JSON object */
                    snprintf(res_buf, sizeof(res_buf),
                             "{\"Update\":{\"uiIconType\":%d,\"uiPower\":%d,\"cpaPower\":%d,"
                             "\"fastCharge\":%d,\"isSupportSpeedCharge\":%d,\"bypassChgStatus\":%d}}",
                             st.ui_icon,
                             st.is_fast ? st.wattage : 0,
                             st.is_fast ? st.wattage : 0,
                             st.is_fast ? 1 : 0,
                             st.sup_speed ? 1 : 0,
                             fast_read_int(NODE_TRAN_BYPASS_CHARGER, 0));
                    break;
                default:
                    strcpy(res_buf, "-1");
                    break;
            }

            AParcel_writeString(out, res_buf, (int32_t)strlen(res_buf));
            return STATUS_OK;
        }

        case 108: { /* testKitFeatureTest */
            AParcel_writeString(out, "unsupported", 11);
            AParcel_writeInt32(out, 0);
            return STATUS_OK;
        }

        /* Versioning & Meta */
        case 16777214: { /* getInterfaceHash */
            AParcel_writeString(out, CHARGER_INTERFACE_HASH, 40);
            return STATUS_OK;
        }

        case 16777215: { /* getInterfaceVersion */
            AParcel_writeInt32(out, CHARGER_INTERFACE_VERSION);
            return STATUS_OK;
        }

        /*
         * String-returning stubs
         */
        case 8:   /* getBattParamNoplug */
        case 13:  /* getBccCsvData */
        case 15:  /* getBmsHeatingRunningStatus */
        case 17:  /* getChargerControl */
        case 23:  /* getDevinfoFastchg */
        case 50:  /* getPsyWirelessRX */
        case 51:  /* getPsyWirelessRxVersion */
        case 52:  /* getPsyWirelessTX */
        case 53:  /* getPsyWirelessTxVersion */
        case 55:  /* getQuickModeGain */
        case 56:  /* getReserveSocDebug */
        case 59:  /* getUisohDebugParameterInfo */
        case 67:  /* getWirelessDeviated */
        case 73:  /* getWirelessTXEnable */
        case 109: /* testKitGetFeatureList */
        case 110: /* testKitGetFeatureName */
        case 113: /* queryWlsPencilInfo */
        case 114: /* getChgOlcConfig */
        case 130: /* getUsbCurrentEyeDiagram */
            AParcel_writeString(out, "", 0);
            return STATUS_OK;

        /*
         * Integer-returning stubs (all remaining transactions)
         */
        default:
            AParcel_writeInt32(out, 0);
            return STATUS_OK;
    }
}

static void *charger_onCreate(void *args) {
    return args;
}

static void charger_onDestroy(void *userData) {
    (void)userData;
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;

    signal(SIGPIPE, SIG_IGN);

    char brand[PROP_VALUE_MAX] = {0};
    char model[PROP_VALUE_MAX] = {0};
    char platform[PROP_VALUE_MAX] = {0};
    __system_property_get("ro.product.brand", brand);
    __system_property_get("ro.product.model", model);
    __system_property_get("ro.board.platform", platform);

    ALOGI("Starting Transsion Charger HAL Adapter for ColorOS (%s %s, SoC: %s, Android 14+)...",
          brand[0] ? brand : "Transsion",
          model[0] ? model : "Device",
          platform[0] ? platform : "Universal");

    /* Initialize global immutable status header once (reusable across all Binder threads, zero heap churn) */
    g_ok_status = AStatus_newOk();

    /* 1. Setup multi-threaded binder threadpool */
    ABinderProcess_setThreadPoolMaxThreadCount(4);
    ABinderProcess_startThreadPool();

    /* 2. Define the AIDL interface class */
    AIBinder_Class *binder_class = AIBinder_Class_define(
        CHARGER_INTERFACE_DESCRIPTOR,
        charger_onCreate,
        charger_onDestroy,
        charger_onTransact
    );

    if (!binder_class) {
        ALOGE("Failed to define AIBinder_Class for %s", CHARGER_INTERFACE_DESCRIPTOR);
        return 1;
    }

    /* 3. Instantiate the binder object */
    AIBinder *service_binder = AIBinder_new(binder_class, NULL);
    if (!service_binder) {
        ALOGE("Failed to allocate AIBinder instance");
        return 1;
    }

    /* 4. Mark VINTF stability (required for AIDL HALs on Android 14+) */
    AIBinder_markVintfStability(service_binder);

    /* 5. Register with ServiceManager */
    binder_status_t status = AServiceManager_addService(service_binder, CHARGER_INSTANCE_NAME);
    if (status != STATUS_OK) {
        ALOGE("Failed to register %s with ServiceManager, status: %d", CHARGER_INSTANCE_NAME, status);
        return 1;
    }

    ALOGI("Successfully registered %s with ServiceManager! Waiting for transactions...", CHARGER_INSTANCE_NAME);

    /* 6. Join threadpool and handle incoming calls indefinitely */
    ABinderProcess_joinThreadPool();

    return 0;
}
