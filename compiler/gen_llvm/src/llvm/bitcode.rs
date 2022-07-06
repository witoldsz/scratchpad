/// Helpers for interacting with the zig that generates bitcode
use crate::debug_info_init;
use crate::llvm::build::{
    complex_bitcast_check_size, load_roc_value, struct_from_fields, to_cc_return, CCReturn, Env,
    C_CALL_CONV, FAST_CALL_CONV, TAG_DATA_INDEX,
};
use crate::llvm::convert::basic_type_from_layout;
use crate::llvm::refcounting::{
    decrement_refcount_layout, increment_n_refcount_layout, increment_refcount_layout,
};
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValue, BasicValueEnum, CallSiteValue, FunctionValue, InstructionValue};
use inkwell::AddressSpace;
use roc_error_macros::internal_error;
use roc_module::symbol::Symbol;
use roc_mono::layout::{LambdaSet, Layout, LayoutIds, UnionLayout};

use super::build::create_entry_block_alloca;

use std::convert::TryInto;

pub fn call_bitcode_fn<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    args: &[BasicValueEnum<'ctx>],
    fn_name: &str,
) -> BasicValueEnum<'ctx> {
    call_bitcode_fn_help(env, args, fn_name)
        .try_as_basic_value()
        .left()
        .unwrap_or_else(|| {
            panic!(
                "LLVM error: Did not get return value from bitcode function {:?}",
                fn_name
            )
        })
}

pub fn call_list_bitcode_fn<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    args: &[BasicValueEnum<'ctx>],
    fn_name: &str,
) -> BasicValueEnum<'ctx> {
    use bumpalo::collections::Vec;

    let parent = env
        .builder
        .get_insert_block()
        .and_then(|b| b.get_parent())
        .unwrap();

    let list_type = super::convert::zig_list_type(env);
    let result = create_entry_block_alloca(env, parent, list_type.into(), "list_alloca");
    let mut arguments: Vec<BasicValueEnum> = Vec::with_capacity_in(args.len() + 1, env.arena);

    arguments.push(result.into());
    arguments.extend(args);

    call_void_bitcode_fn(env, &arguments, fn_name);

    env.builder.build_load(result, "load_list")
}

pub fn call_str_bitcode_fn<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    args: &[BasicValueEnum<'ctx>],
    fn_name: &str,
) -> BasicValueEnum<'ctx> {
    use bumpalo::collections::Vec;

    let parent = env
        .builder
        .get_insert_block()
        .and_then(|b| b.get_parent())
        .unwrap();

    let str_type = super::convert::zig_str_type(env);

    match env.target_info.ptr_width() {
        roc_target::PtrWidth::Bytes4 => {
            // 3 machine words actually fit into 2 registers
            call_bitcode_fn(env, args, fn_name)
        }
        roc_target::PtrWidth::Bytes8 => {
            let result =
                create_entry_block_alloca(env, parent, str_type.into(), "return_str_alloca");
            let mut arguments: Vec<BasicValueEnum> =
                Vec::with_capacity_in(args.len() + 1, env.arena);

            arguments.push(result.into());
            arguments.extend(args);

            call_void_bitcode_fn(env, &arguments, fn_name);

            result.into()
        }
    }
}

pub fn call_void_bitcode_fn<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    args: &[BasicValueEnum<'ctx>],
    fn_name: &str,
) -> InstructionValue<'ctx> {
    call_bitcode_fn_help(env, args, fn_name)
        .try_as_basic_value()
        .right()
        .unwrap_or_else(|| panic!("LLVM error: Tried to call void bitcode function, but got return value from bitcode function, {:?}", fn_name))
}

fn call_bitcode_fn_help<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    args: &[BasicValueEnum<'ctx>],
    fn_name: &str,
) -> CallSiteValue<'ctx> {
    let it = args.iter().map(|x| (*x).into());
    let arguments = bumpalo::collections::Vec::from_iter_in(it, env.arena);

    let fn_val = env
        .module
        .get_function(fn_name)
        .unwrap_or_else(|| panic!("Unrecognized builtin function: {:?} - if you're working on the Roc compiler, do you need to rebuild the bitcode? See compiler/builtins/bitcode/README.md", fn_name));

    let call = env.builder.build_call(fn_val, &arguments, "call_builtin");

    // Attributes that we propagate from the zig builtin parameters, to the arguments we give to the
    // call. It is undefined behavior in LLVM to have an attribute on a parameter, and then call
    // the function where that parameter is not present. For many (e.g. nonnull) it can be inferred
    // but e.g. byval and sret cannot and must be explicitly provided.
    let propagate = [
        Attribute::get_named_enum_kind_id("nonnull"),
        Attribute::get_named_enum_kind_id("nocapture"),
        Attribute::get_named_enum_kind_id("readonly"),
        Attribute::get_named_enum_kind_id("noalias"),
        Attribute::get_named_enum_kind_id("sret"),
        Attribute::get_named_enum_kind_id("byval"),
    ];

    for i in 0..fn_val.count_params() {
        let attributes = fn_val.attributes(AttributeLoc::Param(i));

        for attribute in attributes {
            let kind_id = attribute.get_enum_kind_id();

            if propagate.contains(&kind_id) {
                call.add_attribute(AttributeLoc::Param(i), attribute)
            }
        }
    }

    call.set_call_convention(fn_val.get_call_conventions());
    call
}

pub fn call_bitcode_fn_fixing_for_convention<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    args: &[BasicValueEnum<'ctx>],
    return_layout: &Layout<'_>,
    fn_name: &str,
) -> BasicValueEnum<'ctx> {
    // Calling zig bitcode, so we must follow C calling conventions.
    let cc_return = to_cc_return(env, return_layout);
    match cc_return {
        CCReturn::Return => {
            // We'll get a return value
            call_bitcode_fn(env, args, fn_name)
        }
        CCReturn::ByPointer => {
            // We need to pass the return value by pointer.
            let roc_return_type = basic_type_from_layout(env, return_layout);

            let cc_ptr_return_type = env
                .module
                .get_function(fn_name)
                .unwrap()
                .get_type()
                .get_param_types()[0]
                .into_pointer_type();
            let cc_return_type: BasicTypeEnum<'ctx> = cc_ptr_return_type
                .get_element_type()
                .try_into()
                .expect("Zig bitcode return type is not a basic type!");

            let cc_return_value_ptr = env.builder.build_alloca(cc_return_type, "return_value");
            let fixed_args: Vec<BasicValueEnum<'ctx>> = [cc_return_value_ptr.into()]
                .iter()
                .chain(args)
                .copied()
                .collect();
            call_void_bitcode_fn(env, &fixed_args, fn_name);

            let cc_return_value = env.builder.build_load(cc_return_value_ptr, "read_result");
            if roc_return_type.size_of() == cc_return_type.size_of() {
                cc_return_value
            } else {
                // We need to convert the C-callconv return type, which may be larger than the Roc
                // return type, into the Roc return type.
                complex_bitcast_check_size(
                    env,
                    cc_return_value,
                    roc_return_type,
                    "c_value_to_roc_value",
                )
            }
        }
        CCReturn::Void => {
            internal_error!("Tried to call valued bitcode function, but it has no return type")
        }
    }
}

const ARGUMENT_SYMBOLS: [Symbol; 8] = [
    Symbol::ARG_1,
    Symbol::ARG_2,
    Symbol::ARG_3,
    Symbol::ARG_4,
    Symbol::ARG_5,
    Symbol::ARG_6,
    Symbol::ARG_7,
    Symbol::ARG_8,
];

pub fn build_has_tag_id<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    function: FunctionValue<'ctx>,
    union_layout: UnionLayout<'a>,
) -> FunctionValue<'ctx> {
    let fn_name: &str = &format!("{}_has_tag_id", function.get_name().to_string_lossy());

    // currently the code assumes we're dealing with a non-recursive layout
    debug_assert!(matches!(union_layout, UnionLayout::NonRecursive(_)));

    match env.module.get_function(fn_name) {
        Some(function_value) => function_value,
        None => build_has_tag_id_help(env, union_layout, fn_name),
    }
}

fn build_has_tag_id_help<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    union_layout: UnionLayout<'a>,
    fn_name: &str,
) -> FunctionValue<'ctx> {
    let i8_ptr_type = env.context.i8_type().ptr_type(AddressSpace::Generic);
    let argument_types: &[BasicTypeEnum] = &[env.context.i16_type().into(), i8_ptr_type.into()];

    let block = env.builder.get_insert_block().expect("to be in a function");
    let di_location = env.builder.get_current_debug_location().unwrap();

    let output_type = crate::llvm::convert::zig_has_tag_id_type(env);

    let function_value = crate::llvm::refcounting::build_header_help(
        env,
        fn_name,
        output_type.into(),
        argument_types,
    );

    // called from zig, must use C calling convention
    function_value.set_call_conventions(C_CALL_CONV);

    let kind_id = Attribute::get_named_enum_kind_id("alwaysinline");
    debug_assert!(kind_id > 0);
    let attr = env.context.create_enum_attribute(kind_id, 1);
    function_value.add_attribute(AttributeLoc::Function, attr);

    let entry = env.context.append_basic_block(function_value, "entry");
    env.builder.position_at_end(entry);

    debug_info_init!(env, function_value);

    let it = function_value.get_param_iter();

    let arguments =
        bumpalo::collections::Vec::from_iter_in(it.take(argument_types.len()), env.arena);

    for (argument, name) in arguments.iter().zip(ARGUMENT_SYMBOLS.iter()) {
        argument.set_name(name.as_str(&env.interns));
    }

    match arguments.as_slice() {
        [tag_id, tag_value_ptr] => {
            let tag_type = basic_type_from_layout(env, &Layout::Union(union_layout));

            let tag_value = env.builder.build_pointer_cast(
                tag_value_ptr.into_pointer_value(),
                tag_type.ptr_type(AddressSpace::Generic),
                "load_opaque_get_tag_id",
            );

            let actual_tag_id = {
                let tag_id_i64 = crate::llvm::build::get_tag_id(
                    env,
                    function_value,
                    &union_layout,
                    tag_value.into(),
                );

                env.builder.build_int_cast_sign_flag(
                    tag_id_i64,
                    env.context.i16_type(),
                    true,
                    "to_i16",
                )
            };

            let answer = env.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                tag_id.into_int_value(),
                actual_tag_id,
                "compare",
            );

            let tag_data_ptr = {
                let ptr = env
                    .builder
                    .build_struct_gep(tag_value, TAG_DATA_INDEX, "get_data_ptr")
                    .unwrap();

                env.builder.build_bitcast(ptr, i8_ptr_type, "to_opaque")
            };

            let field_vals = [(0, answer.into()), (1, tag_data_ptr)];

            let output = struct_from_fields(env, output_type, field_vals.iter().copied());

            env.builder.build_return(Some(&output));

            env.builder.position_at_end(block);
            env.builder
                .set_current_debug_location(env.context, di_location);

            function_value
        }
        _ => unreachable!(),
    }
}

pub fn build_transform_caller<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    function: FunctionValue<'ctx>,
    closure_data_layout: LambdaSet<'a>,
    argument_layouts: &[Layout<'a>],
    result_layout: Layout<'a>,
) -> FunctionValue<'ctx> {
    let fn_name: &str = &format!(
        "{}_zig_function_caller",
        function.get_name().to_string_lossy()
    );

    match env.module.get_function(fn_name) {
        Some(function_value) => function_value,
        None => build_transform_caller_help(
            env,
            function,
            closure_data_layout,
            argument_layouts,
            result_layout,
            fn_name,
        ),
    }
}

fn build_transform_caller_help<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    roc_function: FunctionValue<'ctx>,
    closure_data_layout: LambdaSet<'a>,
    argument_layouts: &[Layout<'a>],
    result_layout: Layout<'a>,
    fn_name: &str,
) -> FunctionValue<'ctx> {
    debug_assert!(argument_layouts.len() <= 7);

    let block = env.builder.get_insert_block().expect("to be in a function");
    let di_location = env.builder.get_current_debug_location().unwrap();

    let arg_type = env.context.i8_type().ptr_type(AddressSpace::Generic);

    let function_value = crate::llvm::refcounting::build_header_help(
        env,
        fn_name,
        env.context.void_type().into(),
        &(bumpalo::vec![in env.arena; BasicTypeEnum::PointerType(arg_type); argument_layouts.len() + 2]),
    );

    // called from zig, must use C calling convention
    function_value.set_call_conventions(C_CALL_CONV);

    let kind_id = Attribute::get_named_enum_kind_id("alwaysinline");
    debug_assert!(kind_id > 0);
    let attr = env.context.create_enum_attribute(kind_id, 1);
    function_value.add_attribute(AttributeLoc::Function, attr);

    let entry = env.context.append_basic_block(function_value, "entry");
    env.builder.position_at_end(entry);

    debug_info_init!(env, function_value);

    let mut it = function_value.get_param_iter();
    let closure_ptr = it.next().unwrap().into_pointer_value();
    closure_ptr.set_name(Symbol::ARG_1.as_str(&env.interns));

    let arguments =
        bumpalo::collections::Vec::from_iter_in(it.take(argument_layouts.len()), env.arena);

    for (argument, name) in arguments.iter().zip(ARGUMENT_SYMBOLS[1..].iter()) {
        argument.set_name(name.as_str(&env.interns));
    }

    let mut arguments_cast =
        bumpalo::collections::Vec::with_capacity_in(arguments.len(), env.arena);

    for (argument_ptr, layout) in arguments.iter().zip(argument_layouts) {
        let basic_type = basic_type_from_layout(env, layout).ptr_type(AddressSpace::Generic);

        let argument = if layout.is_passed_by_reference(env.target_info) {
            env.builder
                .build_pointer_cast(
                    argument_ptr.into_pointer_value(),
                    basic_type,
                    "cast_ptr_to_tag_build_transform_caller_help",
                )
                .into()
        } else {
            let argument_cast = env
                .builder
                .build_bitcast(*argument_ptr, basic_type, "load_opaque_1")
                .into_pointer_value();

            env.builder.build_load(argument_cast, "load_opaque_2")
        };

        arguments_cast.push(argument);
    }

    match closure_data_layout.runtime_representation() {
        Layout::Struct {
            field_layouts: &[], ..
        } => {
            // nothing to add
        }
        other => {
            let closure_type = basic_type_from_layout(env, &other).ptr_type(AddressSpace::Generic);

            let closure_cast = env
                .builder
                .build_bitcast(closure_ptr, closure_type, "load_opaque")
                .into_pointer_value();

            let closure_data = env.builder.build_load(closure_cast, "load_opaque");

            arguments_cast.push(closure_data);
        }
    }

    let result = crate::llvm::build::call_roc_function(
        env,
        roc_function,
        &result_layout,
        arguments_cast.as_slice(),
    );

    let result_u8_ptr = function_value
        .get_nth_param(argument_layouts.len() as u32 + 1)
        .unwrap()
        .into_pointer_value();

    crate::llvm::build::store_roc_value_opaque(env, result_layout, result_u8_ptr, result);
    env.builder.build_return(None);

    env.builder.position_at_end(block);
    env.builder
        .set_current_debug_location(env.context, di_location);

    function_value
}

enum Mode {
    Inc,
    IncN,
    Dec,
}

/// a function that accepts two arguments: the value to increment, and an amount to increment by
pub fn build_inc_n_wrapper<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    layout_ids: &mut LayoutIds<'a>,
    layout: &Layout<'a>,
) -> FunctionValue<'ctx> {
    build_rc_wrapper(env, layout_ids, layout, Mode::IncN)
}

/// a function that accepts two arguments: the value to increment; increments by 1
pub fn build_inc_wrapper<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    layout_ids: &mut LayoutIds<'a>,
    layout: &Layout<'a>,
) -> FunctionValue<'ctx> {
    build_rc_wrapper(env, layout_ids, layout, Mode::Inc)
}

pub fn build_dec_wrapper<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    layout_ids: &mut LayoutIds<'a>,
    layout: &Layout<'a>,
) -> FunctionValue<'ctx> {
    build_rc_wrapper(env, layout_ids, layout, Mode::Dec)
}

fn build_rc_wrapper<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    layout_ids: &mut LayoutIds<'a>,
    layout: &Layout<'a>,
    rc_operation: Mode,
) -> FunctionValue<'ctx> {
    let block = env.builder.get_insert_block().expect("to be in a function");
    let di_location = env.builder.get_current_debug_location().unwrap();

    let symbol = Symbol::GENERIC_RC_REF;
    let fn_name = layout_ids
        .get(symbol, layout)
        .to_symbol_string(symbol, &env.interns);

    let fn_name = match rc_operation {
        Mode::IncN => format!("{}_inc_n", fn_name),
        Mode::Inc => format!("{}_inc", fn_name),
        Mode::Dec => format!("{}_dec", fn_name),
    };

    let function_value = match env.module.get_function(fn_name.as_str()) {
        Some(function_value) => function_value,
        None => {
            let arg_type = env.context.i8_type().ptr_type(AddressSpace::Generic);

            let function_value = match rc_operation {
                Mode::Inc | Mode::Dec => crate::llvm::refcounting::build_header_help(
                    env,
                    &fn_name,
                    env.context.void_type().into(),
                    &[arg_type.into()],
                ),
                Mode::IncN => crate::llvm::refcounting::build_header_help(
                    env,
                    &fn_name,
                    env.context.void_type().into(),
                    &[arg_type.into(), env.ptr_int().into()],
                ),
            };

            // called from zig, must use C calling convention
            function_value.set_call_conventions(C_CALL_CONV);

            let kind_id = Attribute::get_named_enum_kind_id("alwaysinline");
            debug_assert!(kind_id > 0);
            let attr = env.context.create_enum_attribute(kind_id, 1);
            function_value.add_attribute(AttributeLoc::Function, attr);

            let entry = env.context.append_basic_block(function_value, "entry");
            env.builder.position_at_end(entry);

            debug_info_init!(env, function_value);

            let mut it = function_value.get_param_iter();
            let value_ptr = it.next().unwrap().into_pointer_value();

            value_ptr.set_name(Symbol::ARG_1.as_str(&env.interns));

            let value_type = basic_type_from_layout(env, layout).ptr_type(AddressSpace::Generic);

            let value = if layout.is_passed_by_reference(env.target_info) {
                env.builder
                    .build_pointer_cast(value_ptr, value_type, "cast_ptr_to_tag_build_rc_wrapper")
                    .into()
            } else {
                let value_cast = env
                    .builder
                    .build_bitcast(value_ptr, value_type, "load_opaque")
                    .into_pointer_value();

                env.builder.build_load(value_cast, "load_opaque")
            };

            match rc_operation {
                Mode::Inc => {
                    let n = 1;
                    increment_refcount_layout(env, function_value, layout_ids, n, value, layout);
                }
                Mode::IncN => {
                    let n = it.next().unwrap().into_int_value();
                    n.set_name(Symbol::ARG_2.as_str(&env.interns));

                    increment_n_refcount_layout(env, function_value, layout_ids, n, value, layout);
                }
                Mode::Dec => {
                    decrement_refcount_layout(env, function_value, layout_ids, value, layout);
                }
            }

            env.builder.build_return(None);

            function_value
        }
    };

    env.builder.position_at_end(block);
    env.builder
        .set_current_debug_location(env.context, di_location);

    function_value
}

pub fn build_eq_wrapper<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    layout_ids: &mut LayoutIds<'a>,
    layout: &Layout<'a>,
) -> FunctionValue<'ctx> {
    let block = env.builder.get_insert_block().expect("to be in a function");
    let di_location = env.builder.get_current_debug_location().unwrap();

    let symbol = Symbol::GENERIC_EQ_REF;
    let fn_name = layout_ids
        .get(symbol, layout)
        .to_symbol_string(symbol, &env.interns);

    let function_value = match env.module.get_function(fn_name.as_str()) {
        Some(function_value) => function_value,
        None => {
            let arg_type = env.context.i8_type().ptr_type(AddressSpace::Generic);

            let function_value = crate::llvm::refcounting::build_header_help(
                env,
                &fn_name,
                env.context.bool_type().into(),
                &[arg_type.into(), arg_type.into()],
            );

            // called from zig, must use C calling convention
            function_value.set_call_conventions(C_CALL_CONV);

            let kind_id = Attribute::get_named_enum_kind_id("alwaysinline");
            debug_assert!(kind_id > 0);
            let attr = env.context.create_enum_attribute(kind_id, 1);
            function_value.add_attribute(AttributeLoc::Function, attr);

            let entry = env.context.append_basic_block(function_value, "entry");
            env.builder.position_at_end(entry);

            debug_info_init!(env, function_value);

            let mut it = function_value.get_param_iter();
            let value_ptr1 = it.next().unwrap().into_pointer_value();
            let value_ptr2 = it.next().unwrap().into_pointer_value();

            value_ptr1.set_name(Symbol::ARG_1.as_str(&env.interns));
            value_ptr2.set_name(Symbol::ARG_2.as_str(&env.interns));

            let value_type = basic_type_from_layout(env, layout).ptr_type(AddressSpace::Generic);

            let value_cast1 = env
                .builder
                .build_bitcast(value_ptr1, value_type, "load_opaque")
                .into_pointer_value();

            let value_cast2 = env
                .builder
                .build_bitcast(value_ptr2, value_type, "load_opaque")
                .into_pointer_value();

            // load_roc_value(env, *element_layout, elem_ptr, "get_elem")
            let value1 = load_roc_value(env, *layout, value_cast1, "load_opaque");
            let value2 = load_roc_value(env, *layout, value_cast2, "load_opaque");

            let result =
                crate::llvm::compare::generic_eq(env, layout_ids, value1, value2, layout, layout);

            env.builder.build_return(Some(&result));

            function_value
        }
    };

    env.builder.position_at_end(block);
    env.builder
        .set_current_debug_location(env.context, di_location);

    function_value
}

pub fn build_compare_wrapper<'a, 'ctx, 'env>(
    env: &Env<'a, 'ctx, 'env>,
    roc_function: FunctionValue<'ctx>,
    closure_data_layout: LambdaSet<'a>,
    layout: &Layout<'a>,
) -> FunctionValue<'ctx> {
    let block = env.builder.get_insert_block().expect("to be in a function");
    let di_location = env.builder.get_current_debug_location().unwrap();

    let fn_name: &str = &format!(
        "{}_compare_wrapper",
        roc_function.get_name().to_string_lossy()
    );

    let function_value = match env.module.get_function(fn_name) {
        Some(function_value) => function_value,
        None => {
            let arg_type = env.context.i8_type().ptr_type(AddressSpace::Generic);

            let function_value = crate::llvm::refcounting::build_header_help(
                env,
                fn_name,
                env.context.i8_type().into(),
                &[arg_type.into(), arg_type.into(), arg_type.into()],
            );

            // called from zig, must use C calling convention
            function_value.set_call_conventions(C_CALL_CONV);

            // we expose this function to zig; must use c calling convention
            function_value.set_call_conventions(C_CALL_CONV);

            let kind_id = Attribute::get_named_enum_kind_id("alwaysinline");
            debug_assert!(kind_id > 0);
            let attr = env.context.create_enum_attribute(kind_id, 1);
            function_value.add_attribute(AttributeLoc::Function, attr);

            let entry = env.context.append_basic_block(function_value, "entry");
            env.builder.position_at_end(entry);

            debug_info_init!(env, function_value);

            let mut it = function_value.get_param_iter();
            let closure_ptr = it.next().unwrap().into_pointer_value();
            let value_ptr1 = it.next().unwrap().into_pointer_value();
            let value_ptr2 = it.next().unwrap().into_pointer_value();

            closure_ptr.set_name(Symbol::ARG_1.as_str(&env.interns));
            value_ptr1.set_name(Symbol::ARG_2.as_str(&env.interns));
            value_ptr2.set_name(Symbol::ARG_3.as_str(&env.interns));

            let value_type = basic_type_from_layout(env, layout);
            let value_ptr_type = value_type.ptr_type(AddressSpace::Generic);

            let value_cast1 = env
                .builder
                .build_bitcast(value_ptr1, value_ptr_type, "load_opaque")
                .into_pointer_value();

            let value_cast2 = env
                .builder
                .build_bitcast(value_ptr2, value_ptr_type, "load_opaque")
                .into_pointer_value();

            let value1 = env.builder.build_load(value_cast1, "load_opaque");
            let value2 = env.builder.build_load(value_cast2, "load_opaque");

            let default = [value1.into(), value2.into()];

            let arguments_cast = match closure_data_layout.runtime_representation() {
                Layout::Struct {
                    field_layouts: &[], ..
                } => {
                    // nothing to add
                    &default
                }
                other => {
                    let closure_type =
                        basic_type_from_layout(env, &other).ptr_type(AddressSpace::Generic);

                    let closure_cast = env
                        .builder
                        .build_bitcast(closure_ptr, closure_type, "load_opaque")
                        .into_pointer_value();

                    let closure_data = env.builder.build_load(closure_cast, "load_opaque");

                    env.arena
                        .alloc([value1.into(), value2.into(), closure_data.into()])
                        as &[_]
                }
            };

            let call = env.builder.build_call(
                roc_function,
                arguments_cast,
                "call_user_defined_compare_function",
            );

            let result = call.try_as_basic_value().left().unwrap();

            // IMPORTANT! we call a user function, so it has the fast calling convention
            call.set_call_convention(FAST_CALL_CONV);

            env.builder.build_return(Some(&result));

            function_value
        }
    };

    env.builder.position_at_end(block);
    env.builder
        .set_current_debug_location(env.context, di_location);

    function_value
}
