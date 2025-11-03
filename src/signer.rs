use std::{
    env,
    ffi::{c_char, c_int, c_longlong, CStr, CString},
    fs,
    path::{Path, PathBuf},
};

use libloading::{Library, Symbol};

use crate::{
    errors::{Result, SignerClientError},
    types::CreateOrderTxReq,
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ApiKeyResponse {
    private_key: *const c_char,
    public_key: *const c_char,
    err: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct RawStrOrErr {
    #[allow(non_snake_case)]
    str_: *const c_char,
    err: *const c_char,
}

type CreateClientFn =
    unsafe extern "C" fn(*const c_char, *const c_char, c_int, c_int, c_longlong) -> *const c_char;
type CheckClientFn = unsafe extern "C" fn(c_int, c_longlong) -> *const c_char;
type SwitchApiKeyFn = unsafe extern "C" fn(c_int) -> *const c_char;
type GenerateApiKeyFn = unsafe extern "C" fn(*const c_char) -> ApiKeyResponse;
type SignChangePubKeyFn = unsafe extern "C" fn(*const c_char, c_longlong) -> RawStrOrErr;
type SignCreateOrderFn = unsafe extern "C" fn(
    c_int,
    c_longlong,
    c_longlong,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_longlong,
    c_longlong,
) -> RawStrOrErr;
type SignCancelOrderFn = unsafe extern "C" fn(c_int, c_longlong, c_longlong) -> RawStrOrErr;
type SignWithdrawFn = unsafe extern "C" fn(c_longlong, c_longlong) -> RawStrOrErr;
type SignCreateSubAccountFn = unsafe extern "C" fn(c_longlong) -> RawStrOrErr;
type SignCancelAllOrdersFn = unsafe extern "C" fn(c_int, c_longlong, c_longlong) -> RawStrOrErr;
type SignModifyOrderFn = unsafe extern "C" fn(
    c_int,
    c_longlong,
    c_longlong,
    c_longlong,
    c_longlong,
    c_longlong,
) -> RawStrOrErr;
type SignTransferFn = unsafe extern "C" fn(
    c_longlong,
    c_longlong,
    c_longlong,
    *const c_char,
    c_longlong,
) -> RawStrOrErr;
type SignCreatePublicPoolFn =
    unsafe extern "C" fn(c_longlong, c_longlong, c_longlong, c_longlong) -> RawStrOrErr;
type SignUpdatePublicPoolFn =
    unsafe extern "C" fn(c_longlong, c_int, c_longlong, c_longlong, c_longlong) -> RawStrOrErr;
type SignMintSharesFn = unsafe extern "C" fn(c_longlong, c_longlong, c_longlong) -> RawStrOrErr;
type SignBurnSharesFn = unsafe extern "C" fn(c_longlong, c_longlong, c_longlong) -> RawStrOrErr;
type SignUpdateLeverageFn = unsafe extern "C" fn(c_int, c_int, c_int, c_longlong) -> RawStrOrErr;
type CreateAuthTokenFn = unsafe extern "C" fn(c_longlong) -> RawStrOrErr;
type SignCreateGroupedOrdersFn = unsafe extern "C" fn(
    u8,                      // grouping_type
    *const CreateOrderTxReq, // orders pointer
    c_int,                   // number of orders
    c_longlong,              // nonce
) -> RawStrOrErr;

pub struct SignerLibrary {
    #[allow(dead_code)]
    lib: Library,
    create_client: CreateClientFn,
    check_client: CheckClientFn,
    switch_api_key: SwitchApiKeyFn,
    generate_api_key: GenerateApiKeyFn,
    sign_change_pub_key: SignChangePubKeyFn,
    sign_create_order: SignCreateOrderFn,
    sign_cancel_order: SignCancelOrderFn,
    sign_withdraw: SignWithdrawFn,
    sign_create_sub_account: SignCreateSubAccountFn,
    sign_cancel_all_orders: SignCancelAllOrdersFn,
    sign_modify_order: SignModifyOrderFn,
    sign_transfer: SignTransferFn,
    sign_create_public_pool: SignCreatePublicPoolFn,
    sign_update_public_pool: SignUpdatePublicPoolFn,
    sign_mint_shares: SignMintSharesFn,
    sign_burn_shares: SignBurnSharesFn,
    sign_update_leverage: SignUpdateLeverageFn,
    create_auth_token: CreateAuthTokenFn,
    sign_create_grouped_orders: SignCreateGroupedOrdersFn,
}

impl SignerLibrary {
    pub fn load_default() -> Result<Self> {
        if let Ok(path) = env::var("LIGHTER_SIGNER_PATH") {
            return Self::load_from_path(Path::new(&path));
        }

        let filename = Self::library_filename()?;
        if let Some(lib) = Self::load_embedded_if_available(&filename)? {
            return Ok(lib);
        }
        let mut candidates = vec![];

        if let Ok(current_dir) = env::current_dir() {
            candidates.push(current_dir.join("signers").join(&filename));
            candidates.push(current_dir.join(&filename));
        }

        if let Ok(exe_path) = env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                candidates.push(dir.join("signers").join(&filename));
                candidates.push(dir.join(&filename));
            }
        }

        for candidate in candidates {
            if candidate.exists() {
                return Self::load_from_path(&candidate);
            }
        }

        Err(SignerClientError::Signer(format!(
            "unable to locate signer library (expected {filename})"
        )))
    }

    fn load_embedded_if_available(filename: &str) -> Result<Option<Self>> {
        if let Some((bytes, expected)) = EMBEDDED_SIGNER {
            if expected == filename {
                return Self::load_from_embedded(bytes, filename).map(Some);
            }
        }
        Ok(None)
    }

    fn load_from_embedded(bytes: &'static [u8], filename: &str) -> Result<Self> {
        let dir = env::temp_dir().join("lighter").join("signers");
        fs::create_dir_all(&dir)?;
        let path = dir.join(filename);
        if !path.exists() {
            fs::write(&path, bytes)?;
        }
        Self::load_from_path(path.as_path())
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let lib = unsafe { Library::new(path) }?;

        macro_rules! load_fn {
            ($sym:expr, $ty:ty) => {{
                let symbol: Symbol<$ty> = unsafe { lib.get($sym)? };
                *symbol
            }};
        }

        let create_client = load_fn!(b"CreateClient\0", CreateClientFn);
        let check_client = load_fn!(b"CheckClient\0", CheckClientFn);
        let switch_api_key = load_fn!(b"SwitchAPIKey\0", SwitchApiKeyFn);
        let generate_api_key = load_fn!(b"GenerateAPIKey\0", GenerateApiKeyFn);
        let sign_change_pub_key = load_fn!(b"SignChangePubKey\0", SignChangePubKeyFn);
        let sign_create_order = load_fn!(b"SignCreateOrder\0", SignCreateOrderFn);
        let sign_cancel_order = load_fn!(b"SignCancelOrder\0", SignCancelOrderFn);
        let sign_withdraw = load_fn!(b"SignWithdraw\0", SignWithdrawFn);
        let sign_create_sub_account = load_fn!(b"SignCreateSubAccount\0", SignCreateSubAccountFn);
        let sign_cancel_all_orders = load_fn!(b"SignCancelAllOrders\0", SignCancelAllOrdersFn);
        let sign_modify_order = load_fn!(b"SignModifyOrder\0", SignModifyOrderFn);
        let sign_transfer = load_fn!(b"SignTransfer\0", SignTransferFn);
        let sign_create_public_pool = load_fn!(b"SignCreatePublicPool\0", SignCreatePublicPoolFn);
        let sign_update_public_pool = load_fn!(b"SignUpdatePublicPool\0", SignUpdatePublicPoolFn);
        let sign_mint_shares = load_fn!(b"SignMintShares\0", SignMintSharesFn);
        let sign_burn_shares = load_fn!(b"SignBurnShares\0", SignBurnSharesFn);
        let sign_update_leverage = load_fn!(b"SignUpdateLeverage\0", SignUpdateLeverageFn);
        let create_auth_token = load_fn!(b"CreateAuthToken\0", CreateAuthTokenFn);
        let sign_create_grouped_orders =
            load_fn!(b"SignCreateGroupedOrders\0", SignCreateGroupedOrdersFn);

        Ok(SignerLibrary {
            lib,
            create_client,
            check_client,
            switch_api_key,
            generate_api_key,
            sign_change_pub_key,
            sign_create_order,
            sign_cancel_order,
            sign_withdraw,
            sign_create_sub_account,
            sign_cancel_all_orders,
            sign_modify_order,
            sign_transfer,
            sign_create_public_pool,
            sign_update_public_pool,
            sign_mint_shares,
            sign_burn_shares,
            sign_update_leverage,
            create_auth_token,
            sign_create_grouped_orders,
        })
    }

    fn library_filename() -> Result<String> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        match (os, arch) {
            ("linux", "x86_64") => Ok("signer-amd64.so".to_string()),
            ("macos", "aarch64") => Ok("signer-arm64.dylib".to_string()),
            _ => Err(SignerClientError::UnsupportedPlatform(format!(
                "{os}/{arch}"
            ))),
        }
    }

    pub fn create_client(
        &self,
        url: &str,
        private_key: &str,
        chain_id: i32,
        api_key_index: i32,
        account_index: i64,
    ) -> Result<Option<String>> {
        let url_c = CString::new(url)?;
        let pk_c = CString::new(private_key)?;
        let err_ptr = unsafe {
            (self.create_client)(
                url_c.as_ptr(),
                pk_c.as_ptr(),
                chain_id,
                api_key_index,
                account_index,
            )
        };

        to_optional_string(err_ptr)
    }

    pub fn check_client(&self, api_key_index: i32, account_index: i64) -> Result<Option<String>> {
        let result = unsafe { (self.check_client)(api_key_index, account_index) };
        to_optional_string(result)
    }

    pub fn switch_api_key(&self, api_key_index: i32) -> Result<Option<String>> {
        let result = unsafe { (self.switch_api_key)(api_key_index) };
        to_optional_string(result)
    }

    pub fn generate_api_key(
        &self,
        seed: Option<&str>,
    ) -> Result<(Option<String>, Option<String>, Option<String>)> {
        let c_seed = CString::new(seed.unwrap_or_default())?;
        let response = unsafe { (self.generate_api_key)(c_seed.as_ptr()) };

        let private_key = to_optional_string(response.private_key)?;
        let public_key = to_optional_string(response.public_key)?;
        let err = to_optional_string(response.err)?;

        Ok((private_key, public_key, err))
    }

    pub fn sign_change_pub_key(
        &self,
        new_pubkey: &str,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let new_pubkey_c = CString::new(new_pubkey)?;
        let raw = unsafe { (self.sign_change_pub_key)(new_pubkey_c.as_ptr(), nonce) };
        str_or_err(raw)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sign_create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: i32,
        time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe {
            (self.sign_create_order)(
                market_index,
                client_order_index,
                base_amount,
                price,
                is_ask as i32,
                order_type,
                time_in_force,
                reduce_only as i32,
                trigger_price,
                order_expiry,
                nonce,
            )
        };
        str_or_err(raw)
    }

    pub fn sign_cancel_order(
        &self,
        market_index: i32,
        order_index: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe { (self.sign_cancel_order)(market_index, order_index, nonce) };
        str_or_err(raw)
    }

    pub fn sign_withdraw(
        &self,
        usdc_amount: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe { (self.sign_withdraw)(usdc_amount, nonce) };
        str_or_err(raw)
    }

    pub fn sign_create_sub_account(&self, nonce: i64) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe { (self.sign_create_sub_account)(nonce) };
        str_or_err(raw)
    }

    pub fn sign_cancel_all_orders(
        &self,
        time_in_force: i32,
        time: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe { (self.sign_cancel_all_orders)(time_in_force, time, nonce) };
        str_or_err(raw)
    }

    pub fn sign_modify_order(
        &self,
        market_index: i32,
        order_index: i64,
        base_amount: i64,
        price: i64,
        trigger_price: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe {
            (self.sign_modify_order)(
                market_index,
                order_index,
                base_amount,
                price,
                trigger_price,
                nonce,
            )
        };
        str_or_err(raw)
    }

    pub fn sign_transfer(
        &self,
        to_account_index: i64,
        usdc_amount: i64,
        fee: i64,
        memo: &str,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let memo_c = CString::new(memo)?;
        let raw = unsafe {
            (self.sign_transfer)(to_account_index, usdc_amount, fee, memo_c.as_ptr(), nonce)
        };
        str_or_err(raw)
    }

    pub fn sign_create_public_pool(
        &self,
        operator_fee: i64,
        initial_total_shares: i64,
        min_operator_share_rate: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe {
            (self.sign_create_public_pool)(
                operator_fee,
                initial_total_shares,
                min_operator_share_rate,
                nonce,
            )
        };
        str_or_err(raw)
    }

    pub fn sign_update_public_pool(
        &self,
        public_pool_index: i64,
        status: i32,
        operator_fee: i64,
        min_operator_share_rate: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe {
            (self.sign_update_public_pool)(
                public_pool_index,
                status,
                operator_fee,
                min_operator_share_rate,
                nonce,
            )
        };
        str_or_err(raw)
    }

    pub fn sign_mint_shares(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe { (self.sign_mint_shares)(public_pool_index, share_amount, nonce) };
        str_or_err(raw)
    }

    pub fn sign_burn_shares(
        &self,
        public_pool_index: i64,
        share_amount: i64,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe { (self.sign_burn_shares)(public_pool_index, share_amount, nonce) };
        str_or_err(raw)
    }

    pub fn sign_update_leverage(
        &self,
        market_index: i32,
        fraction: i32,
        margin_mode: i32,
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw =
            unsafe { (self.sign_update_leverage)(market_index, fraction, margin_mode, nonce) };
        str_or_err(raw)
    }

    pub fn create_auth_token(&self, deadline: i64) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe { (self.create_auth_token)(deadline) };
        str_or_err(raw)
    }

    pub fn sign_create_grouped_orders(
        &self,
        grouping_type: u8,
        orders: &[CreateOrderTxReq],
        nonce: i64,
    ) -> Result<(Option<String>, Option<String>)> {
        let raw = unsafe {
            (self.sign_create_grouped_orders)(
                grouping_type,
                orders.as_ptr(),
                orders.len() as c_int,
                nonce,
            )
        };
        str_or_err(raw)
    }
}

fn to_optional_string(ptr: *const c_char) -> Result<Option<String>> {
    if ptr.is_null() {
        return Ok(None);
    }
    let s = unsafe { CStr::from_ptr(ptr) };
    Ok(Some(s.to_str()?.to_string()))
}

fn str_or_err(raw: RawStrOrErr) -> Result<(Option<String>, Option<String>)> {
    let string = to_optional_string(raw.str_)?;
    let err = to_optional_string(raw.err)?;
    Ok((string, err))
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const EMBEDDED_SIGNER: Option<(&[u8], &str)> = Some((
    include_bytes!("../signers/signer-amd64.so"),
    "signer-amd64.so",
));

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const EMBEDDED_SIGNER: Option<(&[u8], &str)> = Some((
    include_bytes!("../signers/signer-arm64.dylib"),
    "signer-arm64.dylib",
));

#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
)))]
const EMBEDDED_SIGNER: Option<(&[u8], &str)> = None;
