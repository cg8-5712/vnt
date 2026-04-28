use serde::{Serialize, de::DeserializeOwned};
use tauri::{
    Manager, Runtime,
    plugin::TauriPlugin,
};

#[cfg(target_os = "android")]
use tauri::plugin::PluginHandle;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "io.github.cg85712.vnt2.vpn";

pub struct MobileVpn<R: Runtime> {
    #[cfg(not(target_os = "android"))]
    _marker: std::marker::PhantomData<fn() -> R>,
    #[cfg(target_os = "android")]
    mobile_plugin_handle: PluginHandle<R>,
}

impl<R: Runtime> MobileVpn<R> {
    #[cfg(target_os = "android")]
    pub fn run_command<T: DeserializeOwned>(
        &self,
        command: impl AsRef<str>,
        payload: impl Serialize,
    ) -> Result<T, String> {
        self.mobile_plugin_handle
            .run_mobile_plugin(command, payload)
            .map_err(|e| e.to_string())
    }

    #[cfg(not(target_os = "android"))]
    pub fn run_command<T: DeserializeOwned>(
        &self,
        _command: impl AsRef<str>,
        _payload: impl Serialize,
    ) -> Result<T, String> {
        Err("Android VPN service is unavailable on this platform".to_string())
    }
}

pub trait MobileVpnExt<R: Runtime> {
    fn mobile_vpn(&self) -> &MobileVpn<R>;
}

impl<R: Runtime, T: Manager<R>> MobileVpnExt<R> for T {
    fn mobile_vpn(&self) -> &MobileVpn<R> {
        self.state::<MobileVpn<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("mobile-vpn")
        .setup(|app, _api| {
            #[cfg(target_os = "android")]
            let handle = _api.register_android_plugin(PLUGIN_IDENTIFIER, "VntVpnPlugin")?;

            app.manage(MobileVpn {
                #[cfg(not(target_os = "android"))]
                _marker: std::marker::PhantomData::<fn() -> R>,
                #[cfg(target_os = "android")]
                mobile_plugin_handle: handle,
            });

            Ok(())
        })
        .build()
}
