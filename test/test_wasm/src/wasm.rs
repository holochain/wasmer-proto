extern crate test_common;

use holochain_wasmer_guest::*;
use test_common::SomeStruct;
use test_common::StringType;

// define a few functions we expect the host to provide for us
host_externs!(
    __debug,
    __noop,
    __this_func_doesnt_exist_but_we_can_extern_it_anyway,
    __test_process_string,
    __test_process_struct,
    __short_circuit
);

#[no_mangle]
pub extern "C" fn short_circuit(_guest_ptr: GuestPtr, _len: Len) -> GuestPtrLen {
    host_call::<(), String>(__short_circuit, ()).unwrap();
    0
}

#[no_mangle]
pub extern "C" fn literal_bytes(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let bytes: Vec<u8> = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    assert_eq!(bytes, vec![1, 2, 3]);
    return_ptr(bytes)
}

#[no_mangle]
pub extern "C" fn ignore_args_process_string(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    // A well behaved wasm must either use or deallocate the input.
    // A malicious wasm can simply define a __deallocate function that does nothing.
    // The host has no way of knowing whether the guest is behaving right up until it leaks all available memory.
    // If the host tries to force deallocation it risks double-deallocating an honest guest.
    crate::allocation::__deallocate(guest_ptr, len);
    host_call::<&String, StringType>(__test_process_string, &"foo".into()).unwrap();
    return_ptr(StringType::from(String::new()))
}

#[no_mangle]
pub extern "C" fn process_string(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    // get the string the host is trying to pass us out of memory
    // the ptr and cap line up with what was previously allocated with pre_alloc_string
    let s: StringType = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };

    let s: String = format!("guest: {}", String::from(s));
    let s: StringType = try_ptr!(
        host_call::<&String, StringType>(__test_process_string, &s),
        "could not __test_process_string"
    );
    return_ptr(s)
}

#[no_mangle]
pub extern "C" fn process_native(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let input: SomeStruct = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    let processed: SomeStruct = try_ptr!(
        host_call(__test_process_struct, &input),
        "could not deserialize SomeStruct in process_native"
    );
    return_ptr(processed)
}

#[no_mangle]
pub extern "C" fn stacked_strings(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let _: () = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    // get the first string allocated to be returned
    let first = "first";
    // the second string allocation should do nothing to the first
    let _second = "second";

    return_ptr(String::from(first))
}

#[no_mangle]
pub extern "C" fn some_ret(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let _: () = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    return_ptr(SomeStruct::new("foo".into()))
}

#[no_mangle]
pub extern "C" fn some_ret_err(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let _: () = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    return_err_ptr(WasmError::Guest("oh no!".to_string()))
}

#[no_mangle]
pub extern "C" fn native_type(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let input: SomeStruct = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    return_ptr(input)
}

#[no_mangle]
pub extern "C" fn try_ptr_succeeds(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let _: () = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    let ok: Result<SomeStruct, ()> = Ok(SomeStruct::new("foo".into()));
    let result: Result<SomeStruct, ()> = Ok(try_ptr!(ok, "this can't fail"));
    return_ptr(result)
}

#[no_mangle]
pub extern "C" fn try_ptr_fails_fast(guest_ptr: GuestPtr, len: Len) -> GuestPtrLen {
    let _: () = match host_args(guest_ptr, len) {
        Ok(v) => v,
        Err(err_ptr) => return err_ptr,
    };
    let result: Result<(), WasmError> = Ok(try_ptr!(Err(()), "it fails!"));
    return_ptr(result)
}
