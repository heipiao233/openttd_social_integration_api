//! Unofficial Rust binding for OpenTTD Social Integration API.
//! 
//! Use [`openttd_social_integration_api_macros::init`] for entrypoint.
//! 
//! # Examples
//! ```no_run
//! use openttd_social_integration_api::{OpenTTDInfo, PluginApi};
//! 
//! fn shutdown() {
//!     println!("Shutting down!");
//! }
//! 
//! fn run_callbacks() -> bool {
//!     // This println will make log full of garbage.
//!     return true;
//! }
//! 
//! fn event_enter_main_menu() {
//!     println!("Entering main menu!");
//! }
//! 
//! fn event_enter_scenario_editor(map_width: u32, map_height: u32) {
//!     println!("Entering scenario editor ({}x{})!", map_width, map_height);
//! }
//! 
//! fn event_enter_singleplayer(map_width: u32, map_height: u32) {
//!     println!("Entering singleplayer ({}x{})!", map_width, map_height);
//! }
//! 
//! fn event_enter_multiplayer(map_width: u32, map_height: u32) {
//!     println!("Entering multiplayer ({}x{})!", map_width, map_height);
//! }
//! 
//! fn event_joining_multiplayer() {
//!     println!("Joining multiplayer!");
//! }
//! 
//! #[openttd_social_integration_api_macros::init(platform = "test", name = "Test Plugin", version = "0.1")]
//! pub fn init(info: OpenTTDInfo) -> Result<Option<PluginApi>, ()> {
//!     println!("Init for OpenTTD {}", info.openttd_version);
//!     Ok(Some(PluginApi {
//!         shutdown: Some(shutdown),
//!         run_callbacks: Some(run_callbacks),
//!         event_enter_main_menu: Some(event_enter_main_menu),
//!         event_enter_scenario_editor: Some(event_enter_scenario_editor),
//!         event_enter_singleplayer: Some(event_enter_singleplayer),
//!         event_enter_multiplayer: Some(event_enter_multiplayer),
//!         event_joining_multiplayer: Some(event_joining_multiplayer)
//!     }))
//! }
//! 
//! ```

pub mod raw_api;

use std::ffi::CStr;

use crate::raw_api::{OpenTTD_SocialIntegration_v1_InitResult, OpenTTD_SocialIntegration_v1_InitResult_OTTD_SOCIAL_INTEGRATION_V1_INIT_FAILED, OpenTTD_SocialIntegration_v1_InitResult_OTTD_SOCIAL_INTEGRATION_V1_INIT_PLATFORM_NOT_RUNNING, OpenTTD_SocialIntegration_v1_InitResult_OTTD_SOCIAL_INTEGRATION_V1_INIT_SUCCESS, OpenTTD_SocialIntegration_v1_OpenTTDInfo, OpenTTD_SocialIntegration_v1_PluginApi};

/// Pointers supplied by OpenTTD, for the plugin to use.
/// 
/// Package to [raw_api::OpenTTD_SocialIntegration_v1_OpenTTDInfo]
pub struct OpenTTDInfo {
    /// Version of OpenTTD
    pub openttd_version: String
}

impl From<OpenTTD_SocialIntegration_v1_OpenTTDInfo> for OpenTTDInfo {
    fn from(value: OpenTTD_SocialIntegration_v1_OpenTTDInfo) -> Self {
        OpenTTDInfo { openttd_version: unsafe { CStr::from_ptr(value.openttd_version).to_string_lossy().into_owned() } }
    }
}

/// Pointers supplied by the plugin for OpenTTD to use.
/// 
/// Package to [raw_api::OpenTTD_SocialIntegration_v1_PluginApi]
#[derive(Debug, Copy, Clone)]
pub struct PluginApi {
    /// OpenTTD tells the plugin to shut down.
    /// 
    /// The plugin should free any resources it allocated, and must not call any of the callback functions after this call.
    pub shutdown: Option<fn()>,

    /// OpenTTD calls this function at regular intervals, to handle any callbacks the plugin might have.
    /// 
    /// It is also safe to call the OpenTTD_SocialIntegrationCallbacks functions here.
    /// 
    /// If the plugin wants to be called again, please return `true`. Return `false` if the plugin wants to be unloaded.
    pub run_callbacks: Option<fn() -> bool>,

    /// The player has entered the main menu.
    pub event_enter_main_menu: Option<fn()>,
    
    /// The player has entered the Scenario Editor.
    /// 
    /// `map_width` is the width of the map in tiles.
    /// `map_height` is the height of the map in tiles.
    pub event_enter_scenario_editor: Option<fn(map_width: u32, map_height: u32)>,
    /// The player has entered a singleplayer game.
    /// 
    /// `map_width` is the width of the map in tiles.
    /// `map_height` is the height of the map in tiles.
    pub event_enter_singleplayer: Option<fn(map_width: u32, map_height: u32)>,
    /// The player has entered a multiplayer game.
    /// 
    /// `map_width` is the width of the map in tiles.
    /// `map_height` is the height of the map in tiles.
    pub event_enter_multiplayer: Option<fn(map_width: u32, map_height: u32)>,
    /// The player is joining a multiplayer game.
    /// 
    /// This is followed by event_enter_multiplayer() if the join was successful.
    pub event_joining_multiplayer: Option<fn()>,
}

static mut PLUGIN_API: PluginApi = PluginApi {
    shutdown: None,
    run_callbacks: None,
    event_enter_main_menu: None,
    event_enter_scenario_editor: None,
    event_enter_singleplayer: None,
    event_enter_multiplayer: None,
    event_joining_multiplayer: None,
};

unsafe extern "C" fn shutdown() {
    PLUGIN_API.shutdown.unwrap()();
}

unsafe extern "C" fn run_callbacks() -> bool {
    PLUGIN_API.run_callbacks.unwrap()()
}

unsafe extern "C" fn event_enter_main_menu() {
    PLUGIN_API.event_enter_main_menu.unwrap()();
}

unsafe extern "C" fn event_enter_scenario_editor(map_width: u32, map_height: u32) {
    PLUGIN_API.event_enter_scenario_editor.unwrap()(map_width, map_height);
}

unsafe extern "C" fn event_enter_singleplayer(map_width: u32, map_height: u32) {
    PLUGIN_API.event_enter_singleplayer.unwrap()(map_width, map_height);
}

unsafe extern "C" fn event_enter_multiplayer(map_width: u32, map_height: u32) {
    PLUGIN_API.event_enter_multiplayer.unwrap()(map_width, map_height);
}

unsafe extern "C" fn event_joining_multiplayer() {
    PLUGIN_API.event_joining_multiplayer.unwrap()();
}

macro_rules! wrapper_some {
    ($x : ident) => {
        match PLUGIN_API.$x {
            Some(_) => Some($x),
            None => None
        }
    };
}

/// Internal function. Used by proc macro. Don't use in your code.
pub unsafe fn call_init<F> (init: F, info: *const OpenTTD_SocialIntegration_v1_OpenTTDInfo) -> (Option<OpenTTD_SocialIntegration_v1_PluginApi>, OpenTTD_SocialIntegration_v1_InitResult)
    where F: FnOnce(OpenTTDInfo) -> Result<Option<PluginApi>, ()>
{
    match init((*info).into()) {
        Ok(Some(api)) => {
            PLUGIN_API = api;
            (Some(OpenTTD_SocialIntegration_v1_PluginApi {
                shutdown: wrapper_some!(shutdown),
                run_callbacks: wrapper_some!(run_callbacks),
                event_enter_main_menu: wrapper_some!(event_enter_main_menu),
                event_enter_scenario_editor: wrapper_some!(event_enter_scenario_editor),
                event_enter_singleplayer: wrapper_some!(event_enter_singleplayer),
                event_enter_multiplayer: wrapper_some!(event_enter_multiplayer),
                event_joining_multiplayer: wrapper_some!(event_joining_multiplayer),
            }), OpenTTD_SocialIntegration_v1_InitResult_OTTD_SOCIAL_INTEGRATION_V1_INIT_SUCCESS)
        },
        Ok(None) => (None, OpenTTD_SocialIntegration_v1_InitResult_OTTD_SOCIAL_INTEGRATION_V1_INIT_PLATFORM_NOT_RUNNING),
        Err(_) => (None, OpenTTD_SocialIntegration_v1_InitResult_OTTD_SOCIAL_INTEGRATION_V1_INIT_FAILED),
    }
}
