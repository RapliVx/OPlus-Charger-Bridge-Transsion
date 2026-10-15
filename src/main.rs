#![allow(non_snake_case)]

use oplus_charger_bridge_transsion::transsion::TranssionState;
use oplus_charger_bridge_transsion::{INTERFACE_HASH, INTERFACE_VERSION, SERVICE_NAME};

include!(concat!(env!("OUT_DIR"), "/charger.rs"));
use crate::vendor::oplus::hardware::charger::testKitFeatureTestResult::testKitFeatureTestResult;
use crate::vendor::oplus::hardware::charger::ICharger::{BnCharger, ICharger};

type BinderResult<T> = rsbinder::status::Result<T>;

fn init_logging() {
    let _ = tracing_log::LogTracer::init();

    #[cfg(target_os = "android")]
    {
        use tracing_logcat::{LogcatMakeWriter, LogcatTag};
        use tracing_subscriber::fmt::format::Format;

        match LogcatMakeWriter::new(LogcatTag::Fixed("TranChargerHAL".into())) {
            Ok(writer) => {
                let _ = tracing_subscriber::fmt()
                    .event_format(Format::default().without_time())
                    .with_env_filter(
                        tracing_subscriber::EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                    )
                    .with_writer(writer)
                    .with_ansi(false)
                    .try_init();
            }
            Err(_) => {
                let _ = tracing_subscriber::fmt()
                    .with_env_filter(
                        tracing_subscriber::EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                    )
                    .with_ansi(false)
                    .try_init();
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();
}

macro_rules! ok0_methods {
    ($($name:ident $(($($arg:ident: $ty:ty),*))?;)+) => {$(
        #[allow(unused_variables)]
        fn $name(&self $(, $($arg: $ty),*)?) -> BinderResult<i32> { Ok(0) }
    )+};
}

macro_rules! empty_string_methods {
    ($($name:ident $(($($arg:ident: $ty:ty),*))?;)+) => {$(
        #[allow(unused_variables)]
        fn $name(&self $(, $($arg: $ty),*)?) -> BinderResult<String> { Ok(String::new()) }
    )+};
}

struct ChargerService;

impl rsbinder::Interface for ChargerService {}

impl ICharger for ChargerService {
    ok0_methods! {
        VolDividerIcWorkModeSet(data: &str);
        chgExchangeMesgInit;
        chgExchangeSohMesgInit;
        getBattSubCurrent;
        getBccExpStatus;
        getBmsHeatingStatus;
        getChargerCoolDown;
        getChargerCriticalLog;
        getChargerIdVolt;
        getChargerLog;
        getCustomSelectChgMode;
        getParallelChgMosTestResult;
        getPsyBatteryNotify;
        getPsyBatteryPchg;
        getPsyBatteryShortFeature;
        getPsyBatteryShortStatus;
        getPsyOtgOnline;
        getPsyOtgSwitch;
        getPsyPcPortOnline;
        getPsyQGVbatDeviation;
        getPsyTypeOrientation;
        getPsyUsbStatus;
        getQgVbatDeviation;
        getSmartChgMode;
        getUsbPrimalType;
        getWiredOtgOnline;
        getWirelessChargePumpEn;
        getWirelessPenPresent;
        getWirelessPtmcId;
        getWirelessRXEnable;
        getWirelessRealType;
        getWirelessUserSleepMode;
        nightstandby(status: i32);
        notifyScreenStatus(status: i32);
        setChargeEMMode(data: &str);
        setChargerControl(data: &str);
        setChargerCoolDown(data: &str);
        setChargerCriticalLog(data: &str);
        setChargerCycle(data: &str);
        setChargerFactoryModeTest(data: &str);
        setChargerLog(data: &str);
        setChgStatusToBcc(status: i32);
        setCustomSelectChgMode(mode: i32, enable: bool);
        setFastchgFwUpdate(data: &str);
        setPsyMmiChgEn(data: &str);
        setPsyOtgSwitch(data: &str);
        setReserveSocDebug(data: &str);
        setShipMode(data: &str);
        setSmartChgMode(data: &str);
        setSmartCoolDown(coolDown: i32, normalCoolDown: i32, pkgName: &str);
        setTbattPwrOff(data: &str);
        setUisohDebugInfo(data: &str);
        setUsbPrimalType(data: &str);
        setWirelessChargePumpEn(data: &str);
        setWirelessFtmMode(data: &str);
        setWirelessIconDelay(data: &str);
        setWirelessIdtAdcTest(data: &str);
        setWirelessPenSoc(data: &str);
        setWirelessRXEnable(data: &str);
        setWirelessTXEnable(data: &str);
        setWirelessUserSleepMode(data: &str);
        setWlsThirdPartitionInfo(data: &str);
        testKitGetFeatureNum;
        updateUiSohToPartion;
        setChgOlcConfig(data: &str);
        setSuperEnduranceStatus(data: &str);
        setSuperEnduranceCount(data: &str);
        setBobStatus(data: &str);
        setPsySlowChgEn(data: &str);
        getCpVbatDeviation;
        getChargingModeInGsmCall;
        setChargingModeInGsmCall(data: &str);
        setChgRusConfig(data: &str);
        setBatteryLogPush(data: &str);
        setUsbEyeDiagram(model: i32, eyeDiagram: &str, isDefaultEyeDiagram: bool);
    }

    empty_string_methods! {
        getBattParamNoplug;
        getBccCsvData;
        getBmsHeatingRunningStatus;
        getChargerControl;
        getDevinfoFastchg;
        getPsyWirelessRX;
        getPsyWirelessRxVersion;
        getPsyWirelessTX;
        getPsyWirelessTxVersion;
        getReserveSocDebug;
        getWirelessDeviated;
        getWirelessTXEnable;
        queryWlsPencilInfo;
        getChgOlcConfig;
        testKitGetFeatureList;
        testKitGetFeatureName(index: i32);
        getUsbCurrentEyeDiagram(model: i32);
        getQuickModeGain;
        getUisohDebugParameterInfo;
    }

    fn r#getAcType(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(if st.online { 1 } else { 0 })
    }

    fn r#getBattAuthenticate(&self) -> BinderResult<i32> {
        Ok(1) // Genuine battery
    }

    fn r#getPsyBatteryHmac(&self) -> BinderResult<i32> {
        Ok(1) // Genuine battery
    }

    fn r#getBattPPSChgIng(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(if st.is_fast { 1 } else { 0 })
    }

    fn r#getBattPPSChgPower(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(if st.is_fast { st.wattage } else { 0 })
    }

    fn r#getBattShortIcOtpStatus(&self) -> BinderResult<i32> {
        Ok(0)
    }

    fn r#getBattVoocChgIng(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(if st.is_supervooc { 1 } else { 0 })
    }

    fn r#getBatteryVoltageNow(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.voltage_mv)
    }

    fn r#getFastCharge(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(if st.is_fast { 1 } else { 0 })
    }

    fn r#getPsyAcOnline(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(if st.online { 1 } else { 0 })
    }

    fn r#getPsyUsbOnline(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(if st.online { 1 } else { 0 })
    }

    fn r#getPsyBatteryCC(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.charge_counter_mah)
    }

    fn r#getPsyBatteryCurrentNow(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.current_ma)
    }

    fn r#getPsyBatteryFcc(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.charge_full_mah)
    }

    fn r#getPsyBatteryLevel(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.battery_level)
    }

    fn r#getPsyBatteryPchgResetCount(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.cycle_count)
    }

    fn r#getPsyBatteryRm(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.charge_counter_mah)
    }

    fn r#getPsyBatteryStatus(&self) -> BinderResult<String> {
        let st = TranssionState::resolve();
        Ok(st.status)
    }

    fn r#getPsyBatteryTemp(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.temp_tenths)
    }

    fn r#getPsyChargeTech(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.tech)
    }

    fn r#getPsyFastChgType(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.fast_type)
    }

    fn r#getPsyInputCurrent(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.current_ma.abs())
    }

    fn r#getUsbInputCurrentNow(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.current_ma.abs())
    }

    fn r#getUIsohValue(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.soh)
    }

    fn r#getWirelessAdapterPower(&self) -> BinderResult<i32> {
        Ok(0)
    }

    fn r#getWirelessCapacity(&self) -> BinderResult<i32> {
        let st = TranssionState::resolve();
        Ok(st.battery_level)
    }

    fn r#getWirelessCurrentNow(&self) -> BinderResult<i32> {
        Ok(0)
    }

    fn r#getWirelessOnline(&self) -> BinderResult<i32> {
        Ok(0)
    }

    fn r#getWirelessVoltageNow(&self) -> BinderResult<i32> {
        Ok(0)
    }

    fn r#healthd_update_ui_soc_decimal(&self) -> BinderResult<String> {
        let st = TranssionState::resolve();
        Ok(TranssionState::next_decimal_soc(
            st.battery_level,
            st.online,
        ))
    }

    fn r#queryChargeInfo(&self) -> BinderResult<String> {
        let st = TranssionState::resolve();
        Ok(st.build_query_info())
    }

    fn r#getPsyBatterySN(&self) -> BinderResult<String> {
        Ok("MTK_TRANSSION_BATTERY".to_string())
    }

    fn r#getBattGaugeInfo(&self) -> BinderResult<String> {
        Ok("mtk_gauge".to_string())
    }

    fn r#testKitFeatureTest(&self, _index: i32) -> BinderResult<testKitFeatureTestResult> {
        Ok(testKitFeatureTestResult {
            r#str: "unsupported".into(),
            r#ret: 0,
        })
    }

    fn r#getChgConfig(
        &self,
        flag: i32,
        _extra: &str,
        _caller_name: i32,
    ) -> BinderResult<String> {
        let st = TranssionState::resolve();
        let res = match flag {
            0 => st.tech.to_string(), // mChargerTechnology: 3 = SuperVOOC, 2 = VOOC, 0 = Normal
            1 | 2 | 3 => {
                if st.is_fast {
                    st.wattage.to_string()
                } else {
                    "0".to_string()
                }
            }
            4 => {
                // mCPAChargeWattage
                if st.is_fast {
                    st.wattage.to_string()
                } else {
                    "0".to_string()
                }
            }
            5 => st.limit_soc.to_string(),
            6 => (if st.sup_speed { 1 } else { 0 }).to_string(),
            14 => (if st.limit_soc < 100 { 1 } else { 0 }).to_string(),
            20 => st.cycle_count.to_string(),
            21 => "0".to_string(),
            23 => "mtk".to_string(),
            24 | 25 => (if st.bypass_active { 1 } else { 0 }).to_string(),
            30 => st.build_update_json(),
            58 => st.soh.to_string(),
            _ => "-1".to_string(),
        };
        Ok(res)
    }

    fn r#setChgConfig(
        &self,
        flag: i32,
        extra: &str,
        _caller_name: i32,
    ) -> BinderResult<i32> {
        match flag {
            5 => {
                if let Ok(limit) = extra.trim().parse::<i32>() {
                    TranssionState::set_charge_limit(limit);
                }
                Ok(0)
            }
            14 => {
                if extra.trim() == "0" {
                    TranssionState::set_charge_limit(100);
                }
                Ok(0)
            }
            25 => {
                let enabled = extra.contains("switch=1") || extra.trim() == "1";
                TranssionState::set_bypass_charging(enabled);
                Ok(0)
            }
            _ => Ok(-1),
        }
    }

    fn r#getInterfaceVersion(&self) -> BinderResult<i32> {
        Ok(INTERFACE_VERSION)
    }

    fn r#getInterfaceHash(&self) -> BinderResult<String> {
        Ok(INTERFACE_HASH.to_string())
    }
}

fn main() -> anyhow::Result<()> {
    init_logging();

    tracing::info!(
        "Transsion Charger HAL Bridge starting, service: {}",
        SERVICE_NAME
    );

    rsbinder::ProcessState::init_default().map_err(|error| {
        anyhow::anyhow!("failed to initialize Binder process state: {error}")
    })?;
    rsbinder::ProcessState::start_thread_pool();

    let service = ChargerService;
    let binder = BnCharger::new_binder(service);
    rsbinder::hub::add_service(SERVICE_NAME, binder.as_binder())?;

    tracing::info!("Registered. Joining thread pool...");
    rsbinder::ProcessState::join_thread_pool()?;
    Ok(())
}
