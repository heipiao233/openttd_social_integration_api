use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{meta::ParseNestedMeta, parse_macro_input, Error, ItemFn, LitStr, Result};

#[derive(Default)]
struct Attributes {
    social_platform: Option<LitStr>,
    name: Option<LitStr>,
    version: Option<LitStr>,
}

impl Attributes {
    fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("platform") {
            self.social_platform = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("name") {
            self.name = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("version") {
            self.version = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("Unsupported property"))
        }
    }
}

/// Decorates the entrypoint function of a plugin.
/// # Usage
/// This attribute has following 3 argument: platform, name and version.
/// ## Platform
/// The Social Platform this plugin is for.
///
/// As there can only be one plugin active for each Social Platform, this value is used to determine which plugin to use.
///
/// A complete list of names can be found here: <https://wiki.openttd.org/en/Development/Social%20Integration>
///  Please use names from that list, including capitalization.
///
/// If you create a plugin for a new Social Platform, please add it to the wiki page.
///
/// ## Return value for the function
/// Returning `Ok(Some(...))` means the plugin initialized successfully.
///
/// Returning `Ok(None)` means the Social Platform is not running.
///
/// Returning `Err(())` means the plugin failed to initialize (generic error).
/// # Examples
/// ```no_run
/// use openttd_social_integration_api::{PluginApi, OpenTTDInfo};
///
/// #[openttd_social_integration_api_macros::init(platform = "test", name = "Test Plugin", version = "0.1")]
/// pub fn init(info: OpenTTDInfo) -> Result<Option<PluginApi>, ()> {
///     Ok(Some(PluginApi {
///         shutdown: None,
///         run_callbacks: None,
///         event_enter_main_menu: None,
///         event_enter_scenario_editor: None,
///         event_enter_singleplayer: None,
///         event_enter_multiplayer: None,
///         event_joining_multiplayer: None
///     }))
/// }
/// ```
#[proc_macro_attribute]
pub fn init(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemFn);
    let mut attrs = Attributes::default();
    let arg_parser = syn::meta::parser(|meta| attrs.parse(meta));
    let args2 = args.clone();
    parse_macro_input!(args2 with arg_parser);
    if attrs.name.is_none() || attrs.social_platform.is_none() || attrs.version.is_none() {
        return Error::new(
            args.into_iter()
                .next()
                .map_or_else(Span::call_site, |t| t.span())
                .into(),
            "No platform, name or version args!",
        )
        .to_compile_error()
        .into();
    }
    impl_init(attrs, &ast)
}

fn impl_init(attrs: Attributes, ast: &syn::ItemFn) -> TokenStream {
    let name = &ast.sig.ident;
    let platform = attrs.social_platform.unwrap();
    let plugin_name = attrs.name.unwrap();
    let version = attrs.version.unwrap();
    let mut gen = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn SocialIntegration_v1_GetInfo(plugin_info: *mut ::openttd_social_integration_api::raw_api::OpenTTD_SocialIntegration_v1_PluginInfo) {
            *plugin_info = ::openttd_social_integration_api::raw_api::OpenTTD_SocialIntegration_v1_PluginInfo {
                social_platform: stringify!(#platform).as_ptr().cast(),
                name: stringify!(#plugin_name).as_ptr().cast(),
                version: stringify!(#version).as_ptr().cast(),
            };
        }

        #[no_mangle]
        pub unsafe extern "C" fn SocialIntegration_v1_Init(
            plugin_api: *mut ::openttd_social_integration_api::raw_api::OpenTTD_SocialIntegration_v1_PluginApi,
            openttd_info: *const ::openttd_social_integration_api::raw_api::OpenTTD_SocialIntegration_v1_OpenTTDInfo,
        ) -> ::openttd_social_integration_api::raw_api::OpenTTD_SocialIntegration_v1_InitResult {
            let ret = unsafe { ::openttd_social_integration_api::call_init(#name, openttd_info) };
            match ret.0 {
                Some(api) => *plugin_api = api,
                None => {}
            }
            return ret.1;
        }
    };
    gen.extend(ast.to_token_stream());
    gen.into()
}
