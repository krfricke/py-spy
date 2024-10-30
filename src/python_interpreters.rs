/* This code abstracts over different python interpreters by providing
traits for the classes/methods we need and implementations on the bindings objects
from bindgen.

Note this code is unaware of copying memory from the target process, so the
pointer addresses here refer to locations in the target process memory space.
This means we can't dereference them directly.
*/

#![allow(clippy::unnecessary_cast)]

// these bindings are automatically generated by rust bindgen
// using the generate_bindings.py script
use crate::python_bindings::{
    v2_7_15, v3_10_0, v3_11_0, v3_12_0, v3_3_7, v3_5_5, v3_6_6, v3_7_0, v3_8_0, v3_9_5,
};
use crate::utils::offset_of;

pub trait InterpreterState {
    type ThreadState: ThreadState;
    type Object: Object;
    type StringObject: StringObject;
    type ListObject: ListObject;
    type TupleObject: TupleObject;
    fn head(&self) -> *mut Self::ThreadState;
    fn gil_locked(&self) -> Option<bool>;
    fn modules(&self) -> *mut Self::Object;
}

pub trait ThreadState {
    type FrameObject: FrameObject;
    type InterpreterState: InterpreterState;

    fn interp(&self) -> *mut Self::InterpreterState;

    // starting in python 3.11, there is an extra level of indirection
    // in getting the frame. this returns the address
    fn frame_address(&self) -> Option<usize>;

    fn frame(&self, offset: Option<usize>) -> *mut Self::FrameObject;
    fn thread_id(&self) -> u64;
    fn native_thread_id(&self) -> Option<u64>;
    fn next(&self) -> *mut Self;
}

pub trait FrameObject {
    type CodeObject: CodeObject;

    fn code(&self) -> *mut Self::CodeObject;
    fn lasti(&self) -> i32;
    fn back(&self) -> *mut Self;
    fn is_entry(&self) -> bool;
}

pub trait CodeObject {
    type StringObject: StringObject;
    type BytesObject: BytesObject;
    type TupleObject: TupleObject;

    fn name(&self) -> *mut Self::StringObject;
    fn filename(&self) -> *mut Self::StringObject;
    fn line_table(&self) -> *mut Self::BytesObject;
    fn first_lineno(&self) -> i32;
    fn nlocals(&self) -> i32;
    fn argcount(&self) -> i32;
    fn varnames(&self) -> *mut Self::TupleObject;

    fn get_line_number(&self, lasti: i32, table: &[u8]) -> i32;
}

pub trait BytesObject {
    fn size(&self) -> usize;
    fn address(&self, base: usize) -> usize;
}

pub trait StringObject {
    fn ascii(&self) -> bool;
    fn kind(&self) -> u32;
    fn size(&self) -> usize;
    fn address(&self, base: usize) -> usize;
}

pub trait TupleObject {
    fn size(&self) -> usize;
    fn address(&self, base: usize, index: usize) -> usize;
}

pub trait ListObject {
    type Object: Object;
    fn size(&self) -> usize;
    fn item(&self) -> *mut *mut Self::Object;
}

pub trait Object {
    type TypeObject: TypeObject;
    fn ob_type(&self) -> *mut Self::TypeObject;
}

pub trait TypeObject {
    fn name(&self) -> *const ::std::os::raw::c_char;
    fn dictoffset(&self) -> isize;
    fn flags(&self) -> usize;
}

/// This macro provides a common impl for PyThreadState/PyFrameObject/PyCodeObject traits
/// (this code is identical across python versions, we are only abstracting the struct layouts here).
/// String handling changes substantially between python versions, and is handled separately.
macro_rules! PythonCommonImpl {
    ($py: ident, $stringobject: ident) => {
        impl InterpreterState for $py::PyInterpreterState {
            type ThreadState = $py::PyThreadState;
            type Object = $py::PyObject;
            type StringObject = $py::$stringobject;
            type ListObject = $py::PyListObject;
            type TupleObject = $py::PyTupleObject;

            fn head(&self) -> *mut Self::ThreadState {
                self.tstate_head
            }
            fn gil_locked(&self) -> Option<bool> {
                None
            }
            fn modules(&self) -> *mut Self::Object {
                self.modules
            }
        }

        impl ThreadState for $py::PyThreadState {
            type FrameObject = $py::PyFrameObject;
            type InterpreterState = $py::PyInterpreterState;
            fn frame_address(&self) -> Option<usize> {
                None
            }
            fn frame(&self, _: Option<usize>) -> *mut Self::FrameObject {
                self.frame
            }
            fn thread_id(&self) -> u64 {
                self.thread_id as u64
            }
            fn native_thread_id(&self) -> Option<u64> {
                None
            }
            fn next(&self) -> *mut Self {
                self.next
            }
            fn interp(&self) -> *mut Self::InterpreterState {
                self.interp
            }
        }

        impl FrameObject for $py::PyFrameObject {
            type CodeObject = $py::PyCodeObject;

            fn code(&self) -> *mut Self::CodeObject {
                self.f_code
            }
            fn lasti(&self) -> i32 {
                self.f_lasti as i32
            }
            fn back(&self) -> *mut Self {
                self.f_back
            }
            fn is_entry(&self) -> bool {
                true
            }
        }

        impl Object for $py::PyObject {
            type TypeObject = $py::PyTypeObject;
            fn ob_type(&self) -> *mut Self::TypeObject {
                self.ob_type as *mut Self::TypeObject
            }
        }

        impl TypeObject for $py::PyTypeObject {
            fn name(&self) -> *const ::std::os::raw::c_char {
                self.tp_name
            }
            fn dictoffset(&self) -> isize {
                self.tp_dictoffset
            }
            fn flags(&self) -> usize {
                self.tp_flags as usize
            }
        }
    };
}

// We can use this up until python3.10 - where code object lnotab attribute is deprecated
macro_rules! PythonCodeObjectImpl {
    ($py: ident, $bytesobject: ident, $stringobject: ident) => {
        impl CodeObject for $py::PyCodeObject {
            type BytesObject = $py::$bytesobject;
            type StringObject = $py::$stringobject;
            type TupleObject = $py::PyTupleObject;

            fn name(&self) -> *mut Self::StringObject {
                self.co_name as *mut Self::StringObject
            }
            fn filename(&self) -> *mut Self::StringObject {
                self.co_filename as *mut Self::StringObject
            }
            fn line_table(&self) -> *mut Self::BytesObject {
                self.co_lnotab as *mut Self::BytesObject
            }
            fn first_lineno(&self) -> i32 {
                self.co_firstlineno
            }
            fn nlocals(&self) -> i32 {
                self.co_nlocals
            }
            fn argcount(&self) -> i32 {
                self.co_argcount
            }
            fn varnames(&self) -> *mut Self::TupleObject {
                self.co_varnames as *mut Self::TupleObject
            }

            fn get_line_number(&self, lasti: i32, table: &[u8]) -> i32 {
                let lasti = lasti as i32;

                // unpack the line table. format is specified here:
                // https://github.com/python/cpython/blob/3.9/Objects/lnotab_notes.txt
                let size = table.len();
                let mut i = 0;
                let mut line_number: i32 = self.first_lineno();
                let mut bytecode_address: i32 = 0;
                while (i + 1) < size {
                    bytecode_address += i32::from(table[i]);
                    if bytecode_address > lasti {
                        break;
                    }

                    let mut increment = i32::from(table[i + 1]);
                    // Handle negative line increments in the line number table - as shown here:
                    // https://github.com/python/cpython/blob/143a97f6/Objects/lnotab_notes.txt#L48-L49
                    if increment >= 0x80 {
                        increment -= 0x100;
                    }
                    line_number += increment;
                    i += 2;
                }
                line_number
            }
        }
    };
}

fn read_varint(index: &mut usize, table: &[u8]) -> usize {
    let mut ret: usize;
    let mut byte = table[*index];
    let mut shift = 0;
    *index += 1;
    ret = (byte & 63) as usize;

    while byte & 64 != 0 {
        byte = table[*index];
        *index += 1;
        shift += 6;
        ret += ((byte & 63) as usize) << shift;
    }
    ret
}

fn read_signed_varint(index: &mut usize, table: &[u8]) -> isize {
    let unsigned_val = read_varint(index, table);
    if unsigned_val & 1 != 0 {
        -((unsigned_val >> 1) as isize)
    } else {
        (unsigned_val >> 1) as isize
    }
}

// Use for 3.11 and 3.12
macro_rules! CompactCodeObjectImpl {
    ($py: ident, $bytesobject: ident, $stringobject: ident) => {
        impl CodeObject for $py::PyCodeObject {
            type BytesObject = $py::$bytesobject;
            type StringObject = $py::$stringobject;
            type TupleObject = $py::PyTupleObject;

            fn name(&self) -> *mut Self::StringObject {
                self.co_name as *mut Self::StringObject
            }
            fn filename(&self) -> *mut Self::StringObject {
                self.co_filename as *mut Self::StringObject
            }
            fn line_table(&self) -> *mut Self::BytesObject {
                self.co_linetable as *mut Self::BytesObject
            }
            fn first_lineno(&self) -> i32 {
                self.co_firstlineno
            }
            fn nlocals(&self) -> i32 {
                self.co_nlocals
            }
            fn argcount(&self) -> i32 {
                self.co_argcount
            }
            fn varnames(&self) -> *mut Self::TupleObject {
                self.co_localsplusnames as *mut Self::TupleObject
            }

            fn get_line_number(&self, lasti: i32, table: &[u8]) -> i32 {
                // unpack compressed table format from python 3.11
                // https://github.com/python/cpython/pull/91666/files
                let lasti = lasti - offset_of(self, &self.co_code_adaptive) as i32;
                let mut line_number: i32 = self.first_lineno();
                let mut bytecode_address: i32 = 0;

                let mut index: usize = 0;
                loop {
                    if index >= table.len() {
                        break;
                    }
                    let byte = table[index];
                    index += 1;

                    let delta = ((byte & 7) as i32) + 1;
                    bytecode_address += delta * 2;
                    let code = (byte >> 3) & 15;
                    let line_delta = match code {
                        15 => 0,
                        14 => {
                            let delta = read_signed_varint(&mut index, table);
                            read_varint(&mut index, table); // end line
                            read_varint(&mut index, table); // start column
                            read_varint(&mut index, table); // end column
                            delta
                        }
                        13 => read_signed_varint(&mut index, table),
                        10..=12 => {
                            index += 2; // start column / end column
                            (code - 10).into()
                        }
                        _ => {
                            index += 1; // column
                            0
                        }
                    };
                    line_number += line_delta as i32;
                    if bytecode_address >= lasti {
                        break;
                    }
                }
                line_number
            }
        }
    };
}

// String/Byte/List/Tuple handling for Python 3.3+
macro_rules! Python3Impl {
    ($py: ident) => {
        impl BytesObject for $py::PyBytesObject {
            fn size(&self) -> usize {
                self.ob_base.ob_size as usize
            }
            fn address(&self, base: usize) -> usize {
                base + offset_of(self, &self.ob_sval)
            }
        }

        impl StringObject for $py::PyUnicodeObject {
            fn ascii(&self) -> bool {
                self._base._base.state.ascii() != 0
            }
            fn size(&self) -> usize {
                self._base._base.length as usize
            }
            fn kind(&self) -> u32 {
                self._base._base.state.kind()
            }

            fn address(&self, base: usize) -> usize {
                if self._base._base.state.compact() == 0 {
                    return unsafe { self.data.any as usize };
                }

                if self._base._base.state.ascii() == 1 {
                    base + std::mem::size_of::<$py::PyASCIIObject>()
                } else {
                    base + std::mem::size_of::<$py::PyCompactUnicodeObject>()
                }
            }
        }

        impl ListObject for $py::PyListObject {
            type Object = $py::PyObject;
            fn size(&self) -> usize {
                self.ob_base.ob_size as usize
            }
            fn item(&self) -> *mut *mut Self::Object {
                self.ob_item
            }
        }

        impl TupleObject for $py::PyTupleObject {
            fn size(&self) -> usize {
                self.ob_base.ob_size as usize
            }
            fn address(&self, base: usize, index: usize) -> usize {
                base + offset_of(self, &self.ob_item)
                    + index * std::mem::size_of::<*mut $py::PyObject>()
            }
        }
    };
}

// Python 3.12
// TODO: this shares some similarities with python 3.11, we should refactor to a common macro
Python3Impl!(v3_12_0);

impl InterpreterState for v3_12_0::PyInterpreterState {
    type ThreadState = v3_12_0::PyThreadState;
    type Object = v3_12_0::PyObject;
    type StringObject = v3_12_0::PyUnicodeObject;
    type ListObject = v3_12_0::PyListObject;
    type TupleObject = v3_12_0::PyTupleObject;

    fn head(&self) -> *mut Self::ThreadState {
        self.threads.head
    }
    fn gil_locked(&self) -> Option<bool> {
        Some(self._gil.locked._value != 0)
    }

    fn modules(&self) -> *mut Self::Object {
        self.imports.modules
    }
}

impl ThreadState for v3_12_0::PyThreadState {
    type FrameObject = v3_12_0::_PyInterpreterFrame;
    type InterpreterState = v3_12_0::PyInterpreterState;
    fn frame_address(&self) -> Option<usize> {
        // There must be a way to get the offset here without actually creating the object
        let cframe: v3_12_0::_PyCFrame = Default::default();
        let current_frame_offset = offset_of(&cframe, &cframe.current_frame);
        Some(self.cframe as usize + current_frame_offset)
    }
    fn frame(&self, addr: Option<usize>) -> *mut Self::FrameObject {
        addr.unwrap() as *mut Self::FrameObject
    }
    fn thread_id(&self) -> u64 {
        self.thread_id as u64
    }
    fn native_thread_id(&self) -> Option<u64> {
        Some(self.native_thread_id as u64)
    }
    fn next(&self) -> *mut Self {
        self.next
    }
    fn interp(&self) -> *mut Self::InterpreterState {
        self.interp
    }
}

impl FrameObject for v3_12_0::_PyInterpreterFrame {
    type CodeObject = v3_12_0::PyCodeObject;
    fn code(&self) -> *mut Self::CodeObject {
        self.f_code
    }
    fn lasti(&self) -> i32 {
        // this returns the delta from the co_code, but we need to adjust for the
        // offset from co_code.co_code_adaptive. This is slightly easier to do in the
        // get_line_number code, so will adjust there
        let co_code = self.f_code as *const _ as *const u8;
        unsafe { (self.prev_instr as *const u8).offset_from(co_code) as i32 }
    }
    fn back(&self) -> *mut Self {
        self.previous
    }
    fn is_entry(&self) -> bool {
        // https://github.com/python/cpython/pull/108036#issuecomment-1684458828
        const FRAME_OWNED_BY_CSTACK: ::std::os::raw::c_char = 3;
        self.owner == FRAME_OWNED_BY_CSTACK
    }
}

impl Object for v3_12_0::PyObject {
    type TypeObject = v3_12_0::PyTypeObject;
    fn ob_type(&self) -> *mut Self::TypeObject {
        self.ob_type as *mut Self::TypeObject
    }
}

impl TypeObject for v3_12_0::PyTypeObject {
    fn name(&self) -> *const ::std::os::raw::c_char {
        self.tp_name
    }
    fn dictoffset(&self) -> isize {
        self.tp_dictoffset
    }
    fn flags(&self) -> usize {
        self.tp_flags as usize
    }
}

CompactCodeObjectImpl!(v3_12_0, PyBytesObject, PyUnicodeObject);

// Python 3.11
// Python3.11 is sufficiently different from previous versions that we can't use the macros above
// to generate implementations of these traits.
Python3Impl!(v3_11_0);

impl InterpreterState for v3_11_0::PyInterpreterState {
    type ThreadState = v3_11_0::PyThreadState;
    type Object = v3_11_0::PyObject;
    type StringObject = v3_11_0::PyUnicodeObject;
    type ListObject = v3_11_0::PyListObject;
    type TupleObject = v3_11_0::PyTupleObject;
    fn head(&self) -> *mut Self::ThreadState {
        self.threads.head
    }
    fn gil_locked(&self) -> Option<bool> {
        None
    }
    fn modules(&self) -> *mut Self::Object {
        self.modules
    }
}

impl ThreadState for v3_11_0::PyThreadState {
    type FrameObject = v3_11_0::_PyInterpreterFrame;
    type InterpreterState = v3_11_0::PyInterpreterState;
    fn frame_address(&self) -> Option<usize> {
        // There must be a way to get the offset here without actually creating the object
        let cframe: v3_11_0::_PyCFrame = Default::default();
        let current_frame_offset = offset_of(&cframe, &cframe.current_frame);
        Some(self.cframe as usize + current_frame_offset)
    }
    fn frame(&self, addr: Option<usize>) -> *mut Self::FrameObject {
        addr.unwrap() as *mut Self::FrameObject
    }
    fn thread_id(&self) -> u64 {
        self.thread_id as u64
    }
    fn native_thread_id(&self) -> Option<u64> {
        Some(self.native_thread_id as u64)
    }
    fn next(&self) -> *mut Self {
        self.next
    }
    fn interp(&self) -> *mut Self::InterpreterState {
        self.interp
    }
}

impl FrameObject for v3_11_0::_PyInterpreterFrame {
    type CodeObject = v3_11_0::PyCodeObject;
    fn code(&self) -> *mut Self::CodeObject {
        self.f_code
    }
    fn lasti(&self) -> i32 {
        // this returns the delta from the co_code, but we need to adjust for the
        // offset from co_code.co_code_adaptive. This is slightly easier to do in the
        // get_line_number code, so will adjust there
        let co_code = self.f_code as *const _ as *const u8;
        unsafe { (self.prev_instr as *const u8).offset_from(co_code) as i32 }
    }
    fn back(&self) -> *mut Self {
        self.previous
    }
    fn is_entry(&self) -> bool {
        self.is_entry
    }
}

impl Object for v3_11_0::PyObject {
    type TypeObject = v3_11_0::PyTypeObject;
    fn ob_type(&self) -> *mut Self::TypeObject {
        self.ob_type as *mut Self::TypeObject
    }
}

impl TypeObject for v3_11_0::PyTypeObject {
    fn name(&self) -> *const ::std::os::raw::c_char {
        self.tp_name
    }
    fn dictoffset(&self) -> isize {
        self.tp_dictoffset
    }
    fn flags(&self) -> usize {
        self.tp_flags as usize
    }
}

CompactCodeObjectImpl!(v3_11_0, PyBytesObject, PyUnicodeObject);

// Python 3.10
Python3Impl!(v3_10_0);
PythonCommonImpl!(v3_10_0, PyUnicodeObject);

impl CodeObject for v3_10_0::PyCodeObject {
    type BytesObject = v3_10_0::PyBytesObject;
    type StringObject = v3_10_0::PyUnicodeObject;
    type TupleObject = v3_10_0::PyTupleObject;

    fn name(&self) -> *mut Self::StringObject {
        self.co_name as *mut Self::StringObject
    }
    fn filename(&self) -> *mut Self::StringObject {
        self.co_filename as *mut Self::StringObject
    }
    fn line_table(&self) -> *mut Self::BytesObject {
        self.co_linetable as *mut Self::BytesObject
    }
    fn first_lineno(&self) -> i32 {
        self.co_firstlineno
    }
    fn nlocals(&self) -> i32 {
        self.co_nlocals
    }
    fn argcount(&self) -> i32 {
        self.co_argcount
    }
    fn varnames(&self) -> *mut Self::TupleObject {
        self.co_varnames as *mut Self::TupleObject
    }
    fn get_line_number(&self, lasti: i32, table: &[u8]) -> i32 {
        // in Python 3.10 we need to double the lasti instruction value here (and no I don't know why)
        // https://github.com/python/cpython/blob/7b88f63e1dd4006b1a08b9c9f087dd13449ecc76/Python/ceval.c#L5999
        // Whereas in python versions up to 3.9 we didn't.
        // https://github.com/python/cpython/blob/3.9/Python/ceval.c#L4713-L4714
        let lasti = 2 * lasti as i32;

        // unpack the line table. format is specified here:
        // https://github.com/python/cpython/blob/3.10/Objects/lnotab_notes.txt
        let size = table.len();
        let mut i = 0;
        let mut line_number: i32 = self.first_lineno();
        let mut bytecode_address: i32 = 0;
        while (i + 1) < size {
            let delta: u8 = table[i];
            let line_delta: i8 = unsafe { std::mem::transmute(table[i + 1]) };
            i += 2;

            if line_delta == -128 {
                continue;
            }

            line_number += i32::from(line_delta);
            bytecode_address += i32::from(delta);
            if bytecode_address > lasti {
                break;
            }
        }

        line_number
    }
}

// Python 3.9
PythonCommonImpl!(v3_9_5, PyUnicodeObject);
PythonCodeObjectImpl!(v3_9_5, PyBytesObject, PyUnicodeObject);
Python3Impl!(v3_9_5);

// Python 3.8
PythonCommonImpl!(v3_8_0, PyUnicodeObject);
PythonCodeObjectImpl!(v3_8_0, PyBytesObject, PyUnicodeObject);
Python3Impl!(v3_8_0);

// Python 3.7
PythonCommonImpl!(v3_7_0, PyUnicodeObject);
PythonCodeObjectImpl!(v3_7_0, PyBytesObject, PyUnicodeObject);
Python3Impl!(v3_7_0);

// Python 3.6
PythonCommonImpl!(v3_6_6, PyUnicodeObject);
PythonCodeObjectImpl!(v3_6_6, PyBytesObject, PyUnicodeObject);
Python3Impl!(v3_6_6);

// python 3.5 and python 3.4
PythonCommonImpl!(v3_5_5, PyUnicodeObject);
PythonCodeObjectImpl!(v3_5_5, PyBytesObject, PyUnicodeObject);
Python3Impl!(v3_5_5);

// python 3.3
PythonCommonImpl!(v3_3_7, PyUnicodeObject);
PythonCodeObjectImpl!(v3_3_7, PyBytesObject, PyUnicodeObject);
Python3Impl!(v3_3_7);

// Python 2.7
PythonCommonImpl!(v2_7_15, PyStringObject);
PythonCodeObjectImpl!(v2_7_15, PyStringObject, PyStringObject);
impl BytesObject for v2_7_15::PyStringObject {
    fn size(&self) -> usize {
        self.ob_size as usize
    }
    fn address(&self, base: usize) -> usize {
        base + offset_of(self, &self.ob_sval)
    }
}

impl StringObject for v2_7_15::PyStringObject {
    fn ascii(&self) -> bool {
        true
    }
    fn kind(&self) -> u32 {
        1
    }
    fn size(&self) -> usize {
        self.ob_size as usize
    }
    fn address(&self, base: usize) -> usize {
        base + offset_of(self, &self.ob_sval)
    }
}

impl ListObject for v2_7_15::PyListObject {
    type Object = v2_7_15::PyObject;
    fn size(&self) -> usize {
        self.ob_size as usize
    }
    fn item(&self) -> *mut *mut Self::Object {
        self.ob_item
    }
}

impl TupleObject for v2_7_15::PyTupleObject {
    fn size(&self) -> usize {
        self.ob_size as usize
    }
    fn address(&self, base: usize, index: usize) -> usize {
        base + offset_of(self, &self.ob_item)
            + index * std::mem::size_of::<*mut v2_7_15::PyObject>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py3_11_line_numbers() {
        use crate::python_bindings::v3_11_0::PyCodeObject;
        let code = PyCodeObject {
            co_firstlineno: 4,
            ..Default::default()
        };

        let table = [
            128_u8, 0, 221, 4, 8, 132, 74, 136, 118, 209, 4, 22, 212, 4, 22, 208, 4, 22, 208, 4,
            22, 208, 4, 22,
        ];
        assert_eq!(code.get_line_number(214, &table), 5);
    }
}
