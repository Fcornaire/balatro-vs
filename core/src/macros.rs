//! Native -> Lua call helpers
pub mod macros {
    /// Call a global Lua function without arguments
    macro_rules! call_lua_function {
        ($name:expr) => {{
            crate::bridge::call_lua_logged($name, vec![]);
        }};
    }

    macro_rules! execute_lua_function_with_args {
        ($name:expr, $(($arg:expr, $arg_type:ty)),* $(,)?) => {{
            let args: Vec<serde_json::Value> = vec![$({
                let v: $arg_type = $arg;
                crate::bridge::ToLua::to_lua(&v)
            }),*];
            crate::bridge::call_lua_logged($name, args);
        }};
    }

    /// Call and decode the result
    macro_rules! execute_lua_function_with_result {
        ($name:expr, $result_type:ty) => {{
            crate::bridge::call_lua_result::<$result_type>($name)
        }};
    }

    pub(crate) use call_lua_function;
    pub(crate) use execute_lua_function_with_args;
    pub(crate) use execute_lua_function_with_result;
}
