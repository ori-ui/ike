use quote::quote;

#[proc_macro_attribute]
pub fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let ident = &item.sig.ident;

    let expanded = quote! {
        #item

        const _: () = {
            #[unsafe(no_mangle)]
            #[cfg(target_os = "android")]
            extern "C" fn ANativeActivity_onCreate(
                activity: *mut std::ffi::c_void,
                _saved_state: *mut std::ffi::c_void,
                _saved_state_size: usize,
            ) {
                ike::android_main(activity, #ident);
            }
        };
    };

    expanded.into()
}
