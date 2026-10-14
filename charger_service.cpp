/*
 * Copyright (C) 2026 Project EVA / CosFix
 *
 * Transsion Family Charger AIDL HAL Adapter for ColorOS ROMs
 * Target: Tecno / Infinix / Itel (MediaTek MT6789 / Helio G99 / Dimensity)
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

#define CHARGER_INTERFACE_DESCRIPTOR "vendor.oplus.hardware.charger.ICharger"
#define CHARGER_INSTANCE_NAME        "vendor.oplus.hardware.charger.ICharger/default"
#define CHARGER_INTERFACE_HASH       "049782b2370590b7ce37aec3e3070adcd99f96fe"
#define CHARGER_INTERFACE_VERSION    1

/* Transsion & MediaTek Sysfs Nodes */
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
#define NODE_CHARGER_CURRENT_LIMIT   "/sys/class/power_supply/charger/input_current_limit"

#define NODE_TRAN_FAST_CHG           "/sys/devices/platform/charger/fast_chg_indicator"
#define NODE_TRAN_ADAPTER_CAPACITY   "/sys/devices/platform/charger/adapter_capacity"
#define NODE_TRAN_BYPASS_CHARGER     "/sys/devices/platform/charger/bypass_charger"
#define NODE_TRAN_HEALTH_PERCENT     "/sys/devices/platform/charger/battery_health_percent"
#define NODE_TRAN_SC_TUISOC          "/sys/devices/platform/charger/sc_tuisoc"
#define NODE_TRAN_BATTERY_CYCLE      "/sys/devices/platform/charger/tran_battery_cycle"
#define NODE_TRAN_PD_TYPE            "/sys/devices/platform/charger/pd_type"

/* Helpers for safe sysfs reading/writing */
static int read_int(const char *path, int default_val) {
    FILE *fp = fopen(path, "r");
    if (!fp) return default_val;
    int val = default_val;
    if (fscanf(fp, "%d", &val) != 1) {
        val = default_val;
    }
    fclose(fp);
    return val;
}

static void read_string(const char *path, char *buf, size_t max_len, const char *default_val) {
    FILE *fp = fopen(path, "r");
    if (!fp) {
        strncpy(buf, default_val, max_len - 1);
        buf[max_len - 1] = '\0';
        return;
    }
    if (fgets(buf, max_len, fp)) {
        size_t len = strlen(buf);
        while (len > 0 && (buf[len - 1] == '\n' || buf[len - 1] == '\r' || buf[len - 1] == ' ')) {
            buf[len - 1] = '\0';
            len--;
        }
    } else {
        strncpy(buf, default_val, max_len - 1);
        buf[max_len - 1] = '\0';
    }
    fclose(fp);
}

static bool write_string(const char *path, const char *val) {
    FILE *fp = fopen(path, "w");
    if (!fp) {
        ALOGW("Failed to open %s for write", path);
        return false;
    }
    fputs(val, fp);
    fclose(fp);
    return true;
}

static bool write_int(const char *path, int val) {
    char buf[32];
    snprintf(buf, sizeof(buf), "%d", val);
    return write_string(path, buf);
}

/* Transsion Telemetry Helpers */
static inline int get_battery_level(void) {
    return read_int(NODE_BATTERY_CAPACITY, 50);
}

static inline int get_battery_voltage_mv(void) {
    int v = read_int(NODE_BATTERY_VOLTAGE, 3800000);
    return (v > 100000) ? (v / 1000) : v;
}

static inline int get_battery_current_ma(void) {
    int cur = read_int(NODE_BATTERY_CURRENT, 0);
    return (abs(cur) > 10000) ? (cur / 1000) : cur;
}

static inline int get_battery_temp_tenths(void) {
    return read_int(NODE_BATTERY_TEMP, 280);
}

static inline bool is_usb_online(void) {
    if (read_int(NODE_CHARGER_ONLINE, 0) != 0) return true;
    char stat[32] = {0};
    read_string(NODE_BATTERY_STATUS, stat, sizeof(stat), "");
    if (strcmp(stat, "Charging") == 0 || strcmp(stat, "Full") == 0) return true;
    return false;
}

/* Dynamic wattage spoof / detect */
static inline int get_adapter_power_w(void) {
    if (!is_usb_online()) return 0;

    /* 1. System property overrides (optional) */
    char prop_val[PROP_VALUE_MAX] = {0};
    if (__system_property_get("persist.sys.charge_wattage", prop_val) > 0 && prop_val[0] != '\0') {
        int w = atoi(prop_val);
        if (w > 0) return w;
    }
    if (__system_property_get("persist.sys.rianixia.charge-wattage", prop_val) > 0 && prop_val[0] != '\0') {
        int w = atoi(prop_val);
        if (w > 0) return w;
    }
    if (__system_property_get("persist.sys.oplus.charge_wattage", prop_val) > 0 && prop_val[0] != '\0') {
        int w = atoi(prop_val);
        if (w > 0) return w;
    }

    /* 2. Read Transsion hardware adapter capacity node */
    int cap = read_int(NODE_TRAN_ADAPTER_CAPACITY, 0);
    if (cap > 0) return cap;

    /* 3. Tecno Camon 30 default fast charge fallback */
    return 33;
}

static inline int get_battery_soh(void) {
    int soh = read_int(NODE_TRAN_HEALTH_PERCENT, 100);
    return (soh > 0 && soh <= 100) ? soh : 100;
}

static inline int get_cycle_count(void) {
    int cyc = read_int(NODE_TRAN_BATTERY_CYCLE, -1);
    if (cyc >= 0) return cyc;
    return read_int(NODE_BATTERY_CYCLE_COUNT, 0);
}

/* Decimal battery SOC simulation counter for real-time decimal animation */
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

/* Decimal battery SOC simulation counter for real-time decimal animation */
static uint32_t s_decimal_counter = 0;

/*
 * ICharger Binder onTransact handler (130 methods + version/hash)
 */
static binder_status_t charger_onTransact(AIBinder *binder, transaction_code_t code,
                                          const AParcel *in, AParcel *out) {
    (void)binder;

    /* Always start reply with OK status header */
    AStatus *status = AStatus_newOk();
    AParcel_writeStatusHeader(out, status);
    AStatus_delete(status);

    bool online = is_usb_online();
    int wattage = get_adapter_power_w();

    switch (code) {
        /*
         * Active Telemetry & Metric Getters
         */
        case 4: /* getAcType */
            AParcel_writeInt32(out, online ? 1 : 0);
            return STATUS_OK;

        case 5:  /* getBattAuthenticate */
        case 30: /* getPsyBatteryHmac */
            AParcel_writeInt32(out, 1); /* 1 = Genuine Battery */
            return STATUS_OK;

        case 6:  /* getBattPPSChgIng */
            AParcel_writeInt32(out, online ? 1 : 0);
            return STATUS_OK;

        case 7:  /* getBattPPSChgPower */
        case 63: /* getWirelessAdapterPower */
            AParcel_writeInt32(out, online ? wattage : 0);
            return STATUS_OK;

        case 11: /* getBattVoocChgIng - SuperVOOC trigger */
            AParcel_writeInt32(out, online ? 1 : 0);
            return STATUS_OK;

        case 12: /* getBatteryVoltageNow */
            AParcel_writeInt32(out, get_battery_voltage_mv());
            return STATUS_OK;

        case 24: /* getFastCharge - Fast charge active trigger */
            AParcel_writeInt32(out, online ? 1 : 0);
            return STATUS_OK;

        case 26: /* getPsyAcOnline */
            AParcel_writeInt32(out, online ? 1 : 0);
            return STATUS_OK;

        case 27: /* getPsyBatteryCC */
            AParcel_writeInt32(out, read_int(NODE_BATTERY_CHARGE_COUNTER, 2500000) / 1000);
            return STATUS_OK;

        case 28: /* getPsyBatteryCurrentNow */
            AParcel_writeInt32(out, get_battery_current_ma());
            return STATUS_OK;

        case 29: /* getPsyBatteryFcc */
            AParcel_writeInt32(out, read_int(NODE_BATTERY_CHARGE_FULL, 5000000) / 1000);
            return STATUS_OK;

        case 31: /* getPsyBatteryLevel */
        case 64: /* getWirelessCapacity */
            AParcel_writeInt32(out, get_battery_level());
            return STATUS_OK;

        case 35: /* getPsyBatteryRm */
            AParcel_writeInt32(out, read_int(NODE_BATTERY_CHARGE_COUNTER, 2500000) / 1000);
            return STATUS_OK;

        case 38: { /* getPsyBatteryStatus */
            char stat[32];
            read_string(NODE_BATTERY_STATUS, stat, sizeof(stat), online ? "Charging" : "Discharging");
            AParcel_writeString(out, stat, (int32_t)strlen(stat));
            return STATUS_OK;
        }

        case 39: /* getPsyBatteryTemp */
            AParcel_writeInt32(out, get_battery_temp_tenths());
            return STATUS_OK;

        case 40: /* getPsyChargeTech - 3 = SuperVOOC */
            AParcel_writeInt32(out, online ? 3 : 0);
            return STATUS_OK;

        case 41: /* getPsyFastChgType - 2 = SuperVOOC 65W+ */
            AParcel_writeInt32(out, online ? 2 : 0);
            return STATUS_OK;

        case 42: /* getPsyInputCurrent */
        case 60: /* getUsbInputCurrentNow */
            AParcel_writeInt32(out, abs(get_battery_current_ma()));
            return STATUS_OK;

        case 48: /* getPsyUsbOnline */
            AParcel_writeInt32(out, online ? 1 : 0);
            return STATUS_OK;

        case 58: /* getUIsohValue */
            AParcel_writeInt32(out, get_battery_soh());
            return STATUS_OK;

        case 76: { /* healthd_update_ui_soc_decimal - Decimal battery percent animation */
            char dec_soc[16];
            if (online) {
                s_decimal_counter = (s_decimal_counter + 7) % 100;
                snprintf(dec_soc, sizeof(dec_soc), "%d.%02u", get_battery_level(), s_decimal_counter);
            } else {
                snprintf(dec_soc, sizeof(dec_soc), "%d.0", get_battery_level());
            }
            AParcel_writeString(out, dec_soc, (int32_t)strlen(dec_soc));
            return STATUS_OK;
        }

        case 79: { /* queryChargeInfo - ColorOS standard newline-delimited key=value contract */
            char info_buf[2048];
            int level = get_battery_level();
            int vbat = get_battery_voltage_mv();
            int ibat = get_battery_current_ma();
            int temp = get_battery_temp_tenths();
            int cc = read_int(NODE_BATTERY_CHARGE_COUNTER, 2500000) / 1000;
            int soh = get_battery_soh();
            int svooc = online ? 1 : 0;
            int fast_type = online ? 2 : 0;
            int charge_tech = online ? 3 : 0;

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
                level, vbat, temp, ibat, cc, fast_type, svooc, svooc, (online ? wattage : 0), abs(ibat),
                charge_tech, online ? 1 : 2, online ? 1 : 0, soh, (soh == 100) ? 1 : 0, soh
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
            ALOGD("notifyScreenStatus: %d", screen_status);
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
                    enable = (read_int(NODE_TRAN_BYPASS_CHARGER, 0) == 0);
                }
                write_int(NODE_TRAN_BYPASS_CHARGER, enable ? 1 : 0);
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
                write_int(NODE_TRAN_SC_TUISOC, limit_on ? 80 : 100);
                ALOGI("Charge limit hardware node updated: %d", limit_on ? 80 : 100);
            } else if (flag == 5) {
                /* Target SOC Limit */
                int target = 80;
                if (extra_str && extra_str[0] != '\0') {
                    int p = atoi(extra_str);
                    if (p > 0 && p <= 100) target = p;
                }
                write_int(NODE_TRAN_SC_TUISOC, target);
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
                case 0: /* mChargerTechnology: 3 = SuperVOOC, 0 = Normal */
                    snprintf(res_buf, sizeof(res_buf), "%d", online ? 3 : 0);
                    break;
                case 1: /* mChargeWattage */
                case 2:
                case 3:
                case 4: /* mCPAChargeWattage */
                    snprintf(res_buf, sizeof(res_buf), "%d", online ? wattage : 0);
                    break;
                case 5: /* Charge limit percent */
                    snprintf(res_buf, sizeof(res_buf), "%d", read_int(NODE_TRAN_SC_TUISOC, 80));
                    break;
                case 6: /* mIsSupSpeedCharge: 1 = true, 0 = false */
                    snprintf(res_buf, sizeof(res_buf), "%d", online ? 1 : 0);
                    break;
                case 14: /* Limit enabled */
                    snprintf(res_buf, sizeof(res_buf), "%d", (read_int(NODE_TRAN_SC_TUISOC, 100) < 100) ? 1 : 0);
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
                    snprintf(res_buf, sizeof(res_buf), "%d", read_int(NODE_TRAN_BYPASS_CHARGER, 0));
                    break;
                case 30: /* getDataFromHalByUpdate JSON object */
                    snprintf(res_buf, sizeof(res_buf),
                             "{\"Update\":{\"uiIconType\":%d,\"uiPower\":%d,\"cpaPower\":%d,"
                             "\"fastCharge\":%d,\"isSupportSpeedCharge\":%d,\"bypassChgStatus\":%d}}",
                             online ? 2 : 0, online ? wattage : 0, online ? wattage : 0,
                             online ? 1 : 0, online ? 1 : 0, read_int(NODE_TRAN_BYPASS_CHARGER, 0));
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
            AParcel_writeString(out, CHARGER_INTERFACE_HASH, (int32_t)strlen(CHARGER_INTERFACE_HASH));
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

    ALOGI("Starting Transsion Charger HAL Adapter for ColorOS (Android 14+)...");

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
