use crate::codegen::Value;

#[repr(C)]
pub union LyTypeValue{
    pub IntVal: libc::c_int, // 0
    pub DoubleVal: libc::c_double, // 1
    pub BoolVal: bool, // 2
    pub StringVal: *mut libc::c_char, // 3
    pub FunctionVal: *mut LyValue, // 4
    pub ClassVal: *mut LyClass, // 5
    pub DictVal: *mut Map, // 6
    pub ArrayVal: *mut LyArray,  // 7
}

#[repr(C)]
pub struct LyValue{
    pub typeindex: libc::c_short,
    pub val: *mut LyTypeValue,
}

#[repr(C)]
pub struct LyArray{
    pub size: libc::c_int,
    pub max_size: libc::c_int,
    pub values: *mut *mut LyValue,
}

#[repr(C)]
pub struct Map{
    pub size: libc::c_int,
    pub max_size: libc::c_int,
    pub values: *mut *mut libc::c_char,
}

#[repr(C)]
pub struct LyClass {
    pub variables: *mut Map,
    pub methods: *mut Map,
    pub name: *mut libc::c_char,
}

pub fn rust_to_c_lyvalue(rustval: Value) -> LyValue {
    let retval = match rustval {
        Value::Int32(i)=>{
            let lytypeval: *mut LyTypeValue = &mut LyTypeValue{
                IntVal: i
            };
            LyValue{
                typeindex: 0,
                val: lytypeval
            }
        }
        Value::Int64(i)=>{
            let lytypeval: *mut LyTypeValue = &mut LyTypeValue{
                IntVal: i as i32,
            };
            LyValue{
                typeindex: 0,
                val: lytypeval
            }
        }
        Value::Float64(i)=>{
            let lytypeval: *mut LyTypeValue = &mut LyTypeValue{
                DoubleVal: i,
            };
            unsafe{println!("ok float {:?}", (*lytypeval).DoubleVal);}
            LyValue{
                typeindex: 1,
                val: lytypeval
            }
        }
        Value::Float32(i)=>{
            let lytypeval: *mut LyTypeValue = &mut LyTypeValue{
                DoubleVal: i as f64,
            };
            LyValue{
                typeindex: 1,
                val: lytypeval
            }
        }
        Value::Boolean(b)=>{
            let lytypeval: *mut LyTypeValue = &mut LyTypeValue{
                BoolVal: b,
            };
            LyValue{
                typeindex: 2,
                val: lytypeval
            }
        }
        Value::Str(s)=>{
            let lytypeval: *mut LyTypeValue = &mut LyTypeValue{
                StringVal: std::ffi::CString::new(s).unwrap().into_bytes_with_nul().as_mut_ptr() as *mut i8,
            };
            LyValue{
                typeindex: 3,
                val: lytypeval
            }
        }
        _=>panic!("todo..."),
    };
    return retval;
}

pub fn c_to_lyvalue(cval: *mut LyValue) -> Value {
    unsafe{
        match (*cval).typeindex {
            -1=>Value::None,
            0=>Value::Int32((*(*cval).val).IntVal),
            1=>Value::Float64((*(*cval).val).DoubleVal),
            2=>Value::Boolean((*(*cval).val).BoolVal),
            3=>Value::Str(
                core::ffi::CStr::from_ptr(
                    (*(*cval).val).StringVal
                ).to_str().map(|s|s.to_owned()).expect("BLUH")
            ),
            4=>Value::Boolean((*(*cval).val).BoolVal),
            _=>panic!("...")    
        }
    }
}