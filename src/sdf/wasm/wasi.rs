use wasmer::{AsStoreMut, Function, imports};

macro_rules! wasi_func_stub {
    ($name:ident, $($arg:ident: $arg_type:ty),* > $ret_type:ty) => {
        fn $name($($arg: $arg_type),*) -> $ret_type {
            tracing::warn!("WASI function {} (args: {:?}) not implemented, returning 0", stringify!($name), vec![$($arg as i64),*] as Vec<i64>);
            0
        }
    }
}

wasi_func_stub!(args_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(args_sizes_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(environ_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(environ_sizes_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(clock_res_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(clock_time_get, _arg0: i32, _arg1: i32, _arg2: i32 > i32);
wasi_func_stub!(fd_advise, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32 > i32);
wasi_func_stub!(fd_allocate, _arg0: i32, _arg1: i32, _arg2: i32 > i32);
wasi_func_stub!(fd_close, _arg0: i32 > i32);
wasi_func_stub!(fd_datasync, _arg0: i32 > i32);
wasi_func_stub!(fd_fdstat_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(fd_fdstat_set_flags, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(fd_fdstat_set_rights, _arg0: i32, _arg1: i64, _arg2: i64 > i32);
wasi_func_stub!(fd_filestat_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(fd_filestat_set_size, _arg0: i32, _arg1: i64 > i32);
wasi_func_stub!(fd_filestat_set_times, _arg0: i32, _arg1: i64, _arg2: i64, _arg3: i32 > i32);
wasi_func_stub!(fd_pread, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32 > i32);
wasi_func_stub!(fd_prestat_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(fd_prestat_dir_name, _arg0: i32, _arg1: i32, _arg2: i32 > i32);
wasi_func_stub!(fd_pwrite, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32 > i32);
wasi_func_stub!(fd_read, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32 > i32);
wasi_func_stub!(fd_readdir, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i64, _arg4: i32 > i32);
wasi_func_stub!(fd_renumber, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(fd_seek, _arg0: i32, _arg1: i64, _arg2: i32, _arg3: i32 > i32);
wasi_func_stub!(fd_sync, _arg0: i32 > i32);
wasi_func_stub!(fd_tell, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(fd_write, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32 > i32);
wasi_func_stub!(path_create_directory, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(path_filestat_get, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32 > i32);
wasi_func_stub!(path_filestat_set_times, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32, _arg6: i32 > i32);
wasi_func_stub!(path_link, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32, _arg6: i32 > i32);
wasi_func_stub!(path_open, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i64, _arg6: i64, _arg7: i32, _arg8: i32 > i32);
wasi_func_stub!(path_readlink, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32 > i32);
wasi_func_stub!(path_remove_directory, _arg0: i32, _arg1: i32, _arg2: i32 > i32);
wasi_func_stub!(path_rename, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32 > i32);
wasi_func_stub!(path_symlink, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32 > i32);
wasi_func_stub!(path_unlink_file, _arg0: i32, _arg1: i32, _arg2: i32 > i32);
wasi_func_stub!(poll_oneoff, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32 > i32);
wasi_func_stub!(proc_exit, _arg0: i32 > i32);
wasi_func_stub!(proc_raise, _arg0: i32 > i32);
wasi_func_stub!(sched_yield, > i32);
wasi_func_stub!(random_get, _arg0: i32, _arg1: i32 > i32);
wasi_func_stub!(sock_accept, _arg0: i32, _arg1: i32, _arg2: i32 > i32);
wasi_func_stub!(sock_recv, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32, _arg5: i32 > i32);
wasi_func_stub!(sock_send, _arg0: i32, _arg1: i32, _arg2: i32, _arg3: i32, _arg4: i32 > i32);
wasi_func_stub!(sock_shutdown, _arg0: i32, _arg1: i32 > i32);

pub(crate) fn wasi_imports(store: &mut impl AsStoreMut) -> wasmer::Imports {
    imports! {
        "wasi_snapshot_preview1" => {
            "args_get" => Function::new_typed(store, args_get),
            "args_sizes_get" => Function::new_typed(store, args_sizes_get),
            "clock_res_get" => Function::new_typed(store, clock_res_get),
            "clock_time_get" => Function::new_typed(store, clock_time_get),
            "environ_get" => Function::new_typed(store, environ_get),
            "environ_sizes_get" => Function::new_typed(store, environ_sizes_get),
            "fd_advise" => Function::new_typed(store, fd_advise),
            "fd_allocate" => Function::new_typed(store, fd_allocate),
            "fd_close" => Function::new_typed(store, fd_close),
            "fd_datasync" => Function::new_typed(store, fd_datasync),
            "fd_fdstat_get" => Function::new_typed(store, fd_fdstat_get),
            "fd_fdstat_set_flags" => Function::new_typed(store, fd_fdstat_set_flags),
            "fd_fdstat_set_rights" => Function::new_typed(store, fd_fdstat_set_rights),
            "fd_filestat_get" => Function::new_typed(store, fd_filestat_get),
            "fd_filestat_set_size" => Function::new_typed(store, fd_filestat_set_size),
            "fd_filestat_set_times" => Function::new_typed(store, fd_filestat_set_times),
            "fd_pread" => Function::new_typed(store, fd_pread),
            "fd_prestat_get" => Function::new_typed(store, fd_prestat_get),
            "fd_prestat_dir_name" => Function::new_typed(store, fd_prestat_dir_name),
            "fd_pwrite" => Function::new_typed(store, fd_pwrite),
            "fd_read" => Function::new_typed(store, fd_read),
            "fd_readdir" => Function::new_typed(store, fd_readdir),
            "fd_renumber" => Function::new_typed(store, fd_renumber),
            "fd_seek" => Function::new_typed(store, fd_seek),
            "fd_sync" => Function::new_typed(store, fd_sync),
            "fd_tell" => Function::new_typed(store, fd_tell),
            "fd_write" => Function::new_typed(store, fd_write),
            "path_create_directory" => Function::new_typed(store, path_create_directory),
            "path_filestat_get" => Function::new_typed(store, path_filestat_get),
            "path_filestat_set_times" => Function::new_typed(store, path_filestat_set_times),
            "path_link" => Function::new_typed(store, path_link),
            "path_open" => Function::new_typed(store, path_open),
            "path_readlink" => Function::new_typed(store, path_readlink),
            "path_remove_directory" => Function::new_typed(store, path_remove_directory),
            "path_rename" => Function::new_typed(store, path_rename),
            "path_symlink" => Function::new_typed(store, path_symlink),
            "path_unlink_file" => Function::new_typed(store, path_unlink_file),
            "poll_oneoff" => Function::new_typed(store, poll_oneoff),
            "proc_exit" => Function::new_typed(store, proc_exit),
            "proc_raise" => Function::new_typed(store, proc_raise),
            "sched_yield" => Function::new_typed(store, sched_yield),
            "random_get" => Function::new_typed(store, random_get),
            "sock_accept" => Function::new_typed(store, sock_accept),
            "sock_recv" => Function::new_typed(store, sock_recv),
            "sock_send" => Function::new_typed(store, sock_send),
            "sock_shutdown" => Function::new_typed(store, sock_shutdown),
        }
    }
}
