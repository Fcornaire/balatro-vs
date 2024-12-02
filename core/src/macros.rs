pub mod macros {
    macro_rules! call_lua_function {
        ($chunk:expr) => {{
            let lua_states = get_lua_state_ptrs().unwrap().lock().unwrap();
            let state = lua_states[0].load(std::sync::atomic::Ordering::Relaxed);
            unsafe {
                let lua_patcher = LuaPatcher::new(state);
                let res = lua_patcher.call_fn($chunk);
                if let Err(e) = res {
                    error!("[LuaPatcher] Failed to call fn {}: {}", $chunk, e);
                }
            }
        }};
    }

    macro_rules! execute_lua_function_with_args {
        ($fn_to_call:expr, $(($arg:expr, $arg_type:ty)),*) => {{
            let lua_states = get_lua_state_ptrs().unwrap().lock().unwrap();
            let state = lua_states[0].load(std::sync::atomic::Ordering::Relaxed);
            unsafe {
                let lua_patcher = LuaPatcher::new(state);
                let res = lua_patcher.execute_lua_function_with_args::<($($arg_type),*)>($fn_to_call, ($($arg),*));

                if let Err(e) = res {
                    error!("[LuaPatcher] Failed to execute fn {}: {}", $fn_to_call, e);
                }
            }
        }};
    }

    macro_rules! execute_lua_function_with_result {
        ($fn_to_call:expr, $result_type:ty) => {{
            let lua_states = get_lua_state_ptrs().unwrap().lock().unwrap();
            let state = lua_states[0].load(std::sync::atomic::Ordering::Relaxed);
            unsafe {
                let lua_patcher = LuaPatcher::new(state);
                let res = lua_patcher.execute_lua_function_with_result::<$result_type>($fn_to_call);

                if let Err(e) = res.clone() {
                    error!("[LuaPatcher] Failed to call fn {}: {}", $fn_to_call, e);
                }

                res.unwrap()
            }
        }};
    }

    pub(crate) use call_lua_function;
    pub(crate) use execute_lua_function_with_args;
    pub(crate) use execute_lua_function_with_result;
}
