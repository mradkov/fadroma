#[macro_export]
macro_rules! contract {
    (
        $ConfigKey:literal

        $InitMsgType:ident (
            $InitDeps:ident, $InitEnv:ident, $InitMsg:ident : {
                $($InitMsgArg:ident : $InitMsgArgType:ty),*
            }
        ) -> $StateType:ident {
            $($StateKey:ident : $StateKeyType:ty = $StateKeyValue:expr),*
        }

        $QueryMsgEnum:ident (
            $QueryMsg:ident,
            $QueryState:ident,
            $QueryDeps:ident
        ) {
            $($QueryMsgType:ident (
                $($QueryMsgArg:ident : $QueryMsgArgType:ty),*
            ) $QueryMsgHandler:block)*
        }

        $HandleMsgEnum:ident (
            $HandleSender:ident,
            $HandleMsg:ident,
            &mut $HandleState:ident,
            $HandleEnv:ident,
            $HandleDeps:ident
        ) {
            $($HandleMsgType:ident (
                $($HandleMsgArg:ident : $HandleMsgArgType:ty),*
            ) $HandleMsgHandler:block),*
        }

        $ResponseEnum:ident {
            $($Response:ident {
                $($ResponseArg:ident : $ResponseArgType:ty),*
            }),*
        }
    ) => {
        // Contract interface
        pub mod msg {
            use schemars::JsonSchema;
            use serde::{Deserialize, Serialize};
            message!($InitMsgType { $($InitMsgArg: $InitMsgArgType),* });
            $(message!($Response { $($ResponseArg: $ResponseArgType),* });),*
            messages!(
                $QueryMsgEnum  {$($QueryMsgType {$($QueryMsgArg: $QueryMsgArgType),*})*}
                $HandleMsgEnum {$($HandleMsgType {$($HandleMsgArg: $HandleMsgArgType),*})*}
            );
        }

        // Contract implementation
        pub mod contract {
            //state!($ConfigKey, $StateType ($InitDeps, $InitEnv, $InitMsg) {
                //$($StateKey : $StateKeyType = $StateKeyValue),*
            //}, config, config_read);
            use schemars::JsonSchema;
            use serde::{Deserialize, Serialize};
            use cosmwasm_std::Storage;
            use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
            pub static CONFIG_KEY: &[u8] = $ConfigKey;
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
            pub struct $StateType {
                $(pub $StateKey: $StateKeyType),*
            }
            pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, $StateType> {
                singleton(storage, CONFIG_KEY)
            }
            pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, $StateType> {
                singleton_read(storage, CONFIG_KEY)
            }

            use cosmwasm_std::{
                Api, Env, Extern, InitResponse,
                Querier, StdResult, StdError
            };
            pub fn init<S: Storage, A: Api, Q: Querier>(
                $InitDeps: &mut Extern<S, A, Q>,
                $InitEnv:  Env,
                $InitMsg:  crate::msg::InitMsg,
            ) -> StdResult<InitResponse> {
                let state = self::State {
                    $( $StateKey: $StateKeyValue ),*
                };
                self::config(&mut $InitDeps.storage).save(&state)?;
                Ok(InitResponse::default())
            }

            use cosmwasm_std::{to_binary, Binary};
            use super::msg::$QueryMsgEnum;
            pub fn query <S: Storage, A: Api, Q: Querier> (
                $QueryDeps: &Extern<S, A, Q>,
                $QueryMsg:  $QueryMsgEnum,
            ) -> StdResult<Binary> {
                let $QueryState = config_read(&$QueryDeps.storage).load()?;
                match $QueryMsg { $(
                    $QueryMsgEnum::$QueryMsgType { $($QueryMsgArg,)* } => $QueryMsgHandler,
                )* }
            }

            use cosmwasm_std::HandleResponse;
            use super::msg::$HandleMsgEnum;
            pub fn handle <S: Storage, A: Api, Q: Querier> (
                $HandleDeps: &mut Extern<S, A, Q>,
                $HandleEnv:  Env,
                $HandleMsg:  $HandleMsgEnum,
            ) -> StdResult<HandleResponse> {
                match $HandleMsg { $(
                    $HandleMsgEnum::$HandleMsgType { $($HandleMsgArg),* } => {
                        let $HandleSender = $HandleDeps.api.canonical_address(&$HandleEnv.message.sender)?;
                        config(&mut $HandleDeps.storage).update(|mut $HandleState| $HandleMsgHandler)?;
                        Ok(HandleResponse::default())
                    }
                )* }
            }
        }

        // Entry point
        #[cfg(target_arch = "wasm32")]
        mod wasm {
            use super::contract;
            use cosmwasm_std::{ExternalApi, ExternalQuerier, ExternalStorage};
            #[no_mangle] extern "C" fn init (env_ptr: u32, msg_ptr: u32) -> u32 {
                cosmwasm_std::do_init(
                    &contract::init::<ExternalStorage, ExternalApi, ExternalQuerier>,
                    env_ptr, msg_ptr,
                )
            }
            #[no_mangle] extern "C" fn handle (env_ptr: u32, msg_ptr: u32) -> u32 {
                cosmwasm_std::do_handle(
                    &contract::handle::<ExternalStorage, ExternalApi, ExternalQuerier>,
                    env_ptr, msg_ptr,
                )
            }
            #[no_mangle] extern "C" fn query (msg_ptr: u32) -> u32 {
                cosmwasm_std::do_query(
                    &contract::query::<ExternalStorage, ExternalApi, ExternalQuerier>,
                    msg_ptr,
                )
            }
            // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
            // automatically because we `use cosmwasm_std`.
        }
    }
}

#[macro_export]
macro_rules! messages {
    (
        $( $group: ident {
            $($Msg: ident { $( $arg: ident : $type: ty ),* })*
        } )*
    ) => {
        $(
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
            #[serde(rename_all = "snake_case")]
            pub enum $group { $($Msg { $($arg : $type),* }),* }
            $(message!($Msg { $($arg: $type),* }););*
        )* }
}

#[macro_export]
macro_rules! message {
    (
        $Msg: ident { $( $arg: ident: $type: ty ),* }
    ) => {
        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
        pub struct $Msg { $(pub $arg: $type),* }
    }
}
