use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiInfo {
    pub ssid: Option<String>,
    pub signal_dbm: Option<i32>,
    pub channel: Option<u32>,
    pub noise_dbm: Option<i32>,
    pub transmit_rate: Option<f64>,
}

fn sanitize_ssid(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return None;
    }
    let sanitized: String = trimmed
        .chars()
        .filter(|&c| {
            (c.is_alphanumeric() || " .-_#()[]!".contains(c))
                && c != '\u{0000}'
        })
        .collect();
    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

pub struct WifiMonitor;

#[cfg(any(target_os = "windows", test))]
#[derive(Debug)]
struct WindowsWifiSnapshot {
    ssid: Vec<u8>,
    signal_quality: u32,
    channel: Option<u32>,
    tx_rate_kbps: u32,
}

#[cfg(any(target_os = "windows", test))]
fn wifi_info_from_windows_snapshot(snapshot: WindowsWifiSnapshot) -> WifiInfo {
    let ssid = String::from_utf8(snapshot.ssid)
        .ok()
        .and_then(|value| sanitize_ssid(&value));
    let signal_quality = snapshot.signal_quality.min(100);

    WifiInfo {
        ssid,
        signal_dbm: Some((signal_quality as i32 / 2) - 100),
        channel: snapshot.channel,
        noise_dbm: None,
        transmit_rate: (snapshot.tx_rate_kbps > 0)
            .then_some(snapshot.tx_rate_kbps as f64 / 1_000.0),
    }
}

impl WifiMonitor {
    pub fn get_wifi_info() -> WifiInfo {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
                .arg("-I")
                .output();

            match output {
                Ok(result) if result.status.success() => {
                    let text = String::from_utf8_lossy(&result.stdout);
                    return parse_airport_output(&text);
                }
                _ => {}
            }

            let output2 = Command::new("networksetup")
                .args(["-getairportnetwork", "en0"])
                .output();

            match output2 {
                Ok(result) if result.status.success() => {
                    let text = String::from_utf8_lossy(&result.stdout);
                    let ssid = parse_networksetup_ssid(&text);
                    return WifiInfo {
                        ssid,
                        signal_dbm: None,
                        channel: None,
                        noise_dbm: None,
                        transmit_rate: None,
                    };
                }
                _ => {}
            }
        }

        #[cfg(target_os = "windows")]
        if let Some(info) = windows_wifi_info() {
            return info;
        }

        WifiInfo {
            ssid: None,
            signal_dbm: None,
            channel: None,
            noise_dbm: None,
            transmit_rate: None,
        }
    }

    pub fn signal_quality(dbm: i32) -> &'static str {
        match dbm {
            dbm if dbm >= -50 => "Excellent",
            dbm if dbm >= -60 => "Good",
            dbm if dbm >= -70 => "Fair",
            dbm if dbm >= -80 => "Weak",
            _ => "Very Weak",
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_wifi_info() -> Option<WifiInfo> {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};
    use std::slice;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::NetworkManagement::WiFi::{
        wlan_interface_state_connected, wlan_intf_opcode_channel_number,
        wlan_intf_opcode_current_connection, WlanCloseHandle, WlanEnumInterfaces,
        WlanFreeMemory, WlanOpenHandle, WlanQueryInterface, WLAN_CONNECTION_ATTRIBUTES,
        WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST,
    };

    struct ClientHandle(HANDLE);

    impl Drop for ClientHandle {
        fn drop(&mut self) {
            unsafe {
                WlanCloseHandle(self.0, null());
            }
        }
    }

    struct WlanMemory(*mut c_void);

    impl Drop for WlanMemory {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    WlanFreeMemory(self.0);
                }
            }
        }
    }

    unsafe fn query_channel(handle: HANDLE, interface: &WLAN_INTERFACE_INFO) -> Option<u32> {
        let mut size = 0;
        let mut data = null_mut();
        let result = WlanQueryInterface(
            handle,
            &interface.InterfaceGuid,
            wlan_intf_opcode_channel_number,
            null(),
            &mut size,
            &mut data,
            null_mut(),
        );
        if result != 0 || data.is_null() || size < std::mem::size_of::<u32>() as u32 {
            return None;
        }
        let memory = WlanMemory(data);
        let channel = *(memory.0 as *const u32);
        (channel > 0).then_some(channel)
    }

    unsafe {
        let mut negotiated_version = 0;
        let mut raw_handle = null_mut();
        if WlanOpenHandle(2, null(), &mut negotiated_version, &mut raw_handle) != 0
            || raw_handle.is_null()
        {
            return None;
        }
        let handle = ClientHandle(raw_handle);

        let mut raw_list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
        if WlanEnumInterfaces(handle.0, null(), &mut raw_list) != 0 || raw_list.is_null() {
            return None;
        }
        let list_memory = WlanMemory(raw_list.cast());
        let list = &*(list_memory.0 as *const WLAN_INTERFACE_INFO_LIST);
        let interfaces = slice::from_raw_parts(
            list.InterfaceInfo.as_ptr(),
            list.dwNumberOfItems as usize,
        );
        let interface = interfaces
            .iter()
            .find(|item| item.isState == wlan_interface_state_connected)?;

        let mut size = 0;
        let mut raw_connection = null_mut();
        if WlanQueryInterface(
            handle.0,
            &interface.InterfaceGuid,
            wlan_intf_opcode_current_connection,
            null(),
            &mut size,
            &mut raw_connection,
            null_mut(),
        ) != 0
            || raw_connection.is_null()
            || size < std::mem::size_of::<WLAN_CONNECTION_ATTRIBUTES>() as u32
        {
            return None;
        }
        let connection_memory = WlanMemory(raw_connection);
        let connection = &*(connection_memory.0 as *const WLAN_CONNECTION_ATTRIBUTES);
        let association = &connection.wlanAssociationAttributes;
        let ssid_length = (association.dot11Ssid.uSSIDLength as usize)
            .min(association.dot11Ssid.ucSSID.len());

        Some(wifi_info_from_windows_snapshot(WindowsWifiSnapshot {
            ssid: association.dot11Ssid.ucSSID[..ssid_length].to_vec(),
            signal_quality: association.wlanSignalQuality,
            channel: query_channel(handle.0, interface),
            tx_rate_kbps: association.ulTxRate,
        }))
    }
}

#[cfg(target_os = "macos")]
fn parse_airport_output(text: &str) -> WifiInfo {
    let mut ssid = None;
    let mut signal = None;
    let mut channel = None;
    let mut noise = None;
    let mut rate = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim();
            if key == "SSID" {
                ssid = sanitize_ssid(val);
            } else if key == "agrCtlRSSI" {
                signal = val.parse::<i32>().ok();
            } else if key == "agrCtlNoise" {
                noise = val.parse::<i32>().ok();
            } else if key == "channel" {
                channel = val.split(',').next().and_then(|c| c.parse::<u32>().ok());
            } else if key == "lastTxRate" {
                rate = val.parse::<f64>().ok();
            }
        }
    }

    WifiInfo {
        ssid,
        signal_dbm: signal,
        channel,
        noise_dbm: noise,
        transmit_rate: rate,
    }
}

#[cfg(target_os = "macos")]
fn parse_networksetup_ssid(text: &str) -> Option<String> {
    let text = text.trim();
    if text.contains("Current Wi-Fi Network:") || text.contains("You are not associated") {
        text.split(':').nth(1).and_then(|s| sanitize_ssid(s))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{wifi_info_from_windows_snapshot, WindowsWifiSnapshot};

    #[test]
    fn windows_snapshot_maps_signal_and_rates() {
        let info = wifi_info_from_windows_snapshot(WindowsWifiSnapshot {
            ssid: b"Ufficio".to_vec(),
            signal_quality: 78,
            channel: Some(44),
            tx_rate_kbps: 866_000,
        });

        assert_eq!(info.ssid.as_deref(), Some("Ufficio"));
        assert_eq!(info.signal_dbm, Some(-61));
        assert_eq!(info.channel, Some(44));
        assert_eq!(info.transmit_rate, Some(866.0));
    }

    #[test]
    fn windows_snapshot_rejects_invalid_ssid_and_clamps_signal() {
        let info = wifi_info_from_windows_snapshot(WindowsWifiSnapshot {
            ssid: vec![0xff, 0xfe],
            signal_quality: 120,
            channel: None,
            tx_rate_kbps: 0,
        });

        assert_eq!(info.ssid, None);
        assert_eq!(info.signal_dbm, Some(-50));
        assert_eq!(info.transmit_rate, None);
    }
}
