use roc_module::symbol::Symbol;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HigherOrder {
    ListMap {
        xs: Symbol,
    },
    ListMap2 {
        xs: Symbol,
        ys: Symbol,
    },
    ListMap3 {
        xs: Symbol,
        ys: Symbol,
        zs: Symbol,
    },
    ListMap4 {
        xs: Symbol,
        ys: Symbol,
        zs: Symbol,
        ws: Symbol,
    },
    ListMapWithIndex {
        xs: Symbol,
    },
    ListKeepIf {
        xs: Symbol,
    },
    ListWalk {
        xs: Symbol,
        state: Symbol,
    },
    ListWalkUntil {
        xs: Symbol,
        state: Symbol,
    },
    ListWalkBackwards {
        xs: Symbol,
        state: Symbol,
    },
    ListKeepOks {
        xs: Symbol,
    },
    ListKeepErrs {
        xs: Symbol,
    },
    ListSortWith {
        xs: Symbol,
    },
    ListAny {
        xs: Symbol,
    },
    ListAll {
        xs: Symbol,
    },
    ListFindUnsafe {
        xs: Symbol,
    },
    DictWalk {
        xs: Symbol,
        state: Symbol,
    },
}

impl HigherOrder {
    pub fn function_arity(&self) -> usize {
        match self {
            HigherOrder::ListMap { .. } => 1,
            HigherOrder::ListMap2 { .. } => 2,
            HigherOrder::ListMap3 { .. } => 3,
            HigherOrder::ListMap4 { .. } => 4,
            HigherOrder::ListMapWithIndex { .. } => 2,
            HigherOrder::ListKeepIf { .. } => 1,
            HigherOrder::ListWalk { .. } => 2,
            HigherOrder::ListWalkUntil { .. } => 2,
            HigherOrder::ListWalkBackwards { .. } => 2,
            HigherOrder::ListKeepOks { .. } => 1,
            HigherOrder::ListKeepErrs { .. } => 1,
            HigherOrder::ListSortWith { .. } => 2,
            HigherOrder::ListFindUnsafe { .. } => 1,
            HigherOrder::DictWalk { .. } => 2,
            HigherOrder::ListAny { .. } => 1,
            HigherOrder::ListAll { .. } => 1,
        }
    }

    /// Index in the array of arguments of the symbol that is the closure data
    /// (captured environment for this function)
    pub const fn closure_data_index(&self) -> usize {
        use HigherOrder::*;

        match self {
            ListMap { .. }
            | ListMapWithIndex { .. }
            | ListSortWith { .. }
            | ListKeepIf { .. }
            | ListKeepOks { .. }
            | ListKeepErrs { .. }
            | ListAny { .. }
            | ListAll { .. }
            | ListFindUnsafe { .. } => 2,
            ListMap2 { .. } => 3,
            ListMap3 { .. } => 4,
            ListMap4 { .. } => 5,
            ListWalk { .. } | ListWalkUntil { .. } | ListWalkBackwards { .. } | DictWalk { .. } => {
                3
            }
        }
    }

    /// Index of the function symbol in the argument list
    pub const fn function_index(&self) -> usize {
        self.closure_data_index() - 1
    }
}

#[allow(dead_code)]
enum FirstOrder {
    StrConcat,
    StrJoinWith,
    StrIsEmpty,
    StrStartsWith,
    StrStartsWithCodePt,
    StrEndsWith,
    StrSplit,
    StrCountGraphemes,
    StrFromInt,
    StrFromUtf8,
    StrFromUtf8Range,
    StrToUtf8,
    StrRepeat,
    StrFromFloat,
    ListLen,
    ListGetUnsafe,
    ListSet,
    ListSublist,
    ListDropAt,
    ListSingle,
    ListRepeat,
    ListReverse,
    ListConcat,
    ListContains,
    ListAppend,
    ListPrepend,
    ListJoin,
    ListRange,
    ListSwap,
    DictSize,
    DictEmpty,
    DictInsert,
    DictRemove,
    DictContains,
    DictGetUnsafe,
    DictKeys,
    DictValues,
    DictUnion,
    DictIntersection,
    DictDifference,
    SetFromList,
    NumAdd,
    NumAddWrap,
    NumAddChecked,
    NumSub,
    NumSubWrap,
    NumSubChecked,
    NumMul,
    NumMulWrap,
    NumMulChecked,
    NumGt,
    NumGte,
    NumLt,
    NumLte,
    NumCompare,
    NumDivUnchecked,
    NumRemUnchecked,
    NumIsMultipleOf,
    NumAbs,
    NumNeg,
    NumSin,
    NumCos,
    NumSqrtUnchecked,
    NumLogUnchecked,
    NumRound,
    NumToFrac,
    NumPow,
    NumCeiling,
    NumPowInt,
    NumFloor,
    NumIsFinite,
    NumAtan,
    NumAcos,
    NumAsin,
    NumBitwiseAnd,
    NumBitwiseXor,
    NumBitwiseOr,
    NumShiftLeftBy,
    NumShiftRightBy,
    NumBytesToU16,
    NumBytesToU32,
    NumShiftRightZfBy,
    NumIntCast,
    NumFloatCast,
    Eq,
    NotEq,
    And,
    Or,
    Not,
    Hash,
    ExpectTrue,
}
