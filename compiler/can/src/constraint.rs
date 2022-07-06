use crate::abilities::SpecializationId;
use crate::exhaustive::{ExhaustiveContext, SketchedRows};
use crate::expected::{Expected, PExpected};
use roc_collections::soa::{EitherIndex, Index, Slice};
use roc_module::ident::TagName;
use roc_module::symbol::{ModuleId, Symbol};
use roc_region::all::{Loc, Region};
use roc_types::subs::{ExhaustiveMark, IllegalCycleMark, Variable};
use roc_types::types::{Category, PatternCategory, Type};

#[derive(Debug)]
pub struct Constraints {
    pub constraints: Vec<Constraint>,
    pub types: Vec<Type>,
    pub variables: Vec<Variable>,
    pub loc_symbols: Vec<(Symbol, Region)>,
    pub let_constraints: Vec<LetConstraint>,
    pub categories: Vec<Category>,
    pub pattern_categories: Vec<PatternCategory>,
    pub expectations: Vec<Expected<Type>>,
    pub pattern_expectations: Vec<PExpected<Type>>,
    pub includes_tags: Vec<IncludesTag>,
    pub strings: Vec<&'static str>,
    pub sketched_rows: Vec<SketchedRows>,
    pub eq: Vec<Eq>,
    pub pattern_eq: Vec<PatternEq>,
    pub cycles: Vec<Cycle>,
}

impl Default for Constraints {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraints {
    pub fn new() -> Self {
        let constraints = Vec::new();
        let mut types = Vec::new();
        let variables = Vec::new();
        let loc_symbols = Vec::new();
        let let_constraints = Vec::new();
        let mut categories = Vec::with_capacity(16);
        let mut pattern_categories = Vec::with_capacity(16);
        let expectations = Vec::new();
        let pattern_expectations = Vec::new();
        let includes_tags = Vec::new();
        let strings = Vec::new();
        let sketched_rows = Vec::new();
        let eq = Vec::new();
        let pattern_eq = Vec::new();
        let cycles = Vec::new();

        types.extend([
            Type::EmptyRec,
            Type::EmptyTagUnion,
            Type::Apply(Symbol::STR_STR, vec![], Region::zero()),
        ]);

        categories.extend([
            Category::Record,
            Category::ForeignCall,
            Category::OpaqueArg,
            Category::Lambda,
            Category::ClosureSize,
            Category::StrInterpolation,
            Category::If,
            Category::When,
            Category::Float,
            Category::Int,
            Category::Num,
            Category::List,
            Category::Str,
            Category::Character,
        ]);

        pattern_categories.extend([
            PatternCategory::Record,
            PatternCategory::EmptyRecord,
            PatternCategory::PatternGuard,
            PatternCategory::PatternDefault,
            PatternCategory::Set,
            PatternCategory::Map,
            PatternCategory::Str,
            PatternCategory::Num,
            PatternCategory::Int,
            PatternCategory::Float,
            PatternCategory::Character,
        ]);

        Self {
            constraints,
            types,
            variables,
            loc_symbols,
            let_constraints,
            categories,
            pattern_categories,
            expectations,
            pattern_expectations,
            includes_tags,
            strings,
            sketched_rows,
            eq,
            pattern_eq,
            cycles,
        }
    }

    pub const EMPTY_RECORD: Index<Type> = Index::new(0);
    pub const EMPTY_TAG_UNION: Index<Type> = Index::new(1);
    pub const STR: Index<Type> = Index::new(2);

    pub const CATEGORY_RECORD: Index<Category> = Index::new(0);
    pub const CATEGORY_FOREIGNCALL: Index<Category> = Index::new(1);
    pub const CATEGORY_OPAQUEARG: Index<Category> = Index::new(2);
    pub const CATEGORY_LAMBDA: Index<Category> = Index::new(3);
    pub const CATEGORY_CLOSURESIZE: Index<Category> = Index::new(4);
    pub const CATEGORY_STRINTERPOLATION: Index<Category> = Index::new(5);
    pub const CATEGORY_IF: Index<Category> = Index::new(6);
    pub const CATEGORY_WHEN: Index<Category> = Index::new(7);
    pub const CATEGORY_FLOAT: Index<Category> = Index::new(8);
    pub const CATEGORY_INT: Index<Category> = Index::new(9);
    pub const CATEGORY_NUM: Index<Category> = Index::new(10);
    pub const CATEGORY_LIST: Index<Category> = Index::new(11);
    pub const CATEGORY_STR: Index<Category> = Index::new(12);
    pub const CATEGORY_CHARACTER: Index<Category> = Index::new(13);

    pub const PCATEGORY_RECORD: Index<PatternCategory> = Index::new(0);
    pub const PCATEGORY_EMPTYRECORD: Index<PatternCategory> = Index::new(1);
    pub const PCATEGORY_PATTERNGUARD: Index<PatternCategory> = Index::new(2);
    pub const PCATEGORY_PATTERNDEFAULT: Index<PatternCategory> = Index::new(3);
    pub const PCATEGORY_SET: Index<PatternCategory> = Index::new(4);
    pub const PCATEGORY_MAP: Index<PatternCategory> = Index::new(5);
    pub const PCATEGORY_STR: Index<PatternCategory> = Index::new(6);
    pub const PCATEGORY_NUM: Index<PatternCategory> = Index::new(7);
    pub const PCATEGORY_INT: Index<PatternCategory> = Index::new(8);
    pub const PCATEGORY_FLOAT: Index<PatternCategory> = Index::new(9);
    pub const PCATEGORY_CHARACTER: Index<PatternCategory> = Index::new(10);

    #[inline(always)]
    pub fn push_type(&mut self, typ: Type) -> EitherIndex<Type, Variable> {
        match typ {
            Type::EmptyRec => EitherIndex::from_left(Self::EMPTY_RECORD),
            Type::EmptyTagUnion => EitherIndex::from_left(Self::EMPTY_TAG_UNION),
            Type::Apply(Symbol::STR_STR, args, _) if args.is_empty() => {
                EitherIndex::from_left(Self::STR)
            }
            Type::Variable(var) => Self::push_type_variable(var),
            other => {
                let index: Index<Type> = Index::push_new(&mut self.types, other);
                EitherIndex::from_left(index)
            }
        }
    }

    pub fn statistics(&self, module_id: ModuleId) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;

        let mut buf = String::new();

        writeln!(buf, "Constraints statistics for module {:?}:", module_id)?;

        writeln!(buf, "   constraints length: {}:", self.constraints.len())?;
        writeln!(buf, "   types length: {}:", self.types.len())?;
        writeln!(
            buf,
            "   let_constraints length: {}:",
            self.let_constraints.len()
        )?;
        writeln!(buf, "   expectations length: {}:", self.expectations.len())?;
        writeln!(buf, "   categories length: {}:", self.categories.len())?;

        Ok(buf)
    }

    #[inline(always)]
    const fn push_type_variable(var: Variable) -> EitherIndex<Type, Variable> {
        // that's right, we use the variable's integer value as the index
        // that way, we don't need to push anything onto a vector
        let index: Index<Variable> = Index::new(var.index());

        EitherIndex::from_right(index)
    }

    #[inline(always)]
    pub fn push_expected_type(&mut self, expected: Expected<Type>) -> Index<Expected<Type>> {
        Index::push_new(&mut self.expectations, expected)
    }

    #[inline(always)]
    pub fn push_category(&mut self, category: Category) -> Index<Category> {
        match category {
            Category::Record => Self::CATEGORY_RECORD,
            Category::ForeignCall => Self::CATEGORY_FOREIGNCALL,
            Category::OpaqueArg => Self::CATEGORY_OPAQUEARG,
            Category::Lambda => Self::CATEGORY_LAMBDA,
            Category::ClosureSize => Self::CATEGORY_CLOSURESIZE,
            Category::StrInterpolation => Self::CATEGORY_STRINTERPOLATION,
            Category::If => Self::CATEGORY_IF,
            Category::When => Self::CATEGORY_WHEN,
            Category::Float => Self::CATEGORY_FLOAT,
            Category::Int => Self::CATEGORY_INT,
            Category::Num => Self::CATEGORY_NUM,
            Category::List => Self::CATEGORY_LIST,
            Category::Str => Self::CATEGORY_STR,
            Category::Character => Self::CATEGORY_CHARACTER,
            other => Index::push_new(&mut self.categories, other),
        }
    }

    #[inline(always)]
    pub fn push_pattern_category(&mut self, category: PatternCategory) -> Index<PatternCategory> {
        match category {
            PatternCategory::Record => Self::PCATEGORY_RECORD,
            PatternCategory::EmptyRecord => Self::PCATEGORY_EMPTYRECORD,
            PatternCategory::PatternGuard => Self::PCATEGORY_PATTERNGUARD,
            PatternCategory::PatternDefault => Self::PCATEGORY_PATTERNDEFAULT,
            PatternCategory::Set => Self::PCATEGORY_SET,
            PatternCategory::Map => Self::PCATEGORY_MAP,
            PatternCategory::Str => Self::PCATEGORY_STR,
            PatternCategory::Num => Self::PCATEGORY_NUM,
            PatternCategory::Int => Self::PCATEGORY_INT,
            PatternCategory::Float => Self::PCATEGORY_FLOAT,
            PatternCategory::Character => Self::PCATEGORY_CHARACTER,
            other => Index::push_new(&mut self.pattern_categories, other),
        }
    }

    #[inline(always)]
    pub fn equal_types(
        &mut self,
        typ: Type,
        expected: Expected<Type>,
        category: Category,
        region: Region,
    ) -> Constraint {
        let type_index = self.push_type(typ);
        let expected_index = Index::push_new(&mut self.expectations, expected);
        let category_index = Self::push_category(self, category);

        Constraint::Eq(Eq(type_index, expected_index, category_index, region))
    }

    #[inline(always)]
    pub fn equal_types_var(
        &mut self,
        var: Variable,
        expected: Expected<Type>,
        category: Category,
        region: Region,
    ) -> Constraint {
        let type_index = Self::push_type_variable(var);
        let expected_index = Index::push_new(&mut self.expectations, expected);
        let category_index = Self::push_category(self, category);

        Constraint::Eq(Eq(type_index, expected_index, category_index, region))
    }

    #[inline(always)]
    pub fn equal_types_with_storage(
        &mut self,
        typ: Type,
        expected: Expected<Type>,
        category: Category,
        region: Region,
        storage_var: Variable,
    ) -> Constraint {
        let type_index = self.push_type(typ);
        let expected_index = Index::push_new(&mut self.expectations, expected);
        let category_index = Self::push_category(self, category);

        let equal = Constraint::Eq(Eq(type_index, expected_index, category_index, region));

        let storage_type_index = Self::push_type_variable(storage_var);
        let storage_category = Category::Storage(std::file!(), std::line!());
        let storage_category_index = Self::push_category(self, storage_category);
        let storage = Constraint::Eq(Eq(
            storage_type_index,
            expected_index,
            storage_category_index,
            region,
        ));

        self.and_constraint([equal, storage])
    }

    pub fn equal_pattern_types(
        &mut self,
        typ: Type,
        expected: PExpected<Type>,
        category: PatternCategory,
        region: Region,
    ) -> Constraint {
        let type_index = self.push_type(typ);
        let expected_index = Index::push_new(&mut self.pattern_expectations, expected);
        let category_index = Self::push_pattern_category(self, category);

        Constraint::Pattern(type_index, expected_index, category_index, region)
    }

    pub fn pattern_presence(
        &mut self,
        typ: Type,
        expected: PExpected<Type>,
        category: PatternCategory,
        region: Region,
    ) -> Constraint {
        let type_index = self.push_type(typ);
        let expected_index = Index::push_new(&mut self.pattern_expectations, expected);
        let category_index = Index::push_new(&mut self.pattern_categories, category);

        Constraint::PatternPresence(type_index, expected_index, category_index, region)
    }

    pub fn is_open_type(&mut self, typ: Type) -> Constraint {
        let type_index = self.push_type(typ);

        Constraint::IsOpenType(type_index)
    }

    pub fn includes_tag<I>(
        &mut self,
        typ: Type,
        tag_name: TagName,
        types: I,
        category: PatternCategory,
        region: Region,
    ) -> Constraint
    where
        I: IntoIterator<Item = Type>,
    {
        let type_index = Index::push_new(&mut self.types, typ);
        let category_index = Index::push_new(&mut self.pattern_categories, category);
        let types_slice = Slice::extend_new(&mut self.types, types);

        let includes_tag = IncludesTag {
            type_index,
            tag_name,
            types: types_slice,
            pattern_category: category_index,
            region,
        };

        let includes_tag_index = Index::push_new(&mut self.includes_tags, includes_tag);

        Constraint::IncludesTag(includes_tag_index)
    }

    fn variable_slice<I>(&mut self, it: I) -> Slice<Variable>
    where
        I: IntoIterator<Item = Variable>,
    {
        let start = self.variables.len();
        self.variables.extend(it);
        let length = self.variables.len() - start;

        Slice::new(start as _, length as _)
    }

    fn def_types_slice<I>(&mut self, it: I) -> DefTypes
    where
        I: IntoIterator<Item = (Symbol, Loc<Type>)>,
        I::IntoIter: ExactSizeIterator,
    {
        let it = it.into_iter();

        let types_start = self.types.len();
        let loc_symbols_start = self.loc_symbols.len();

        // because we have an ExactSizeIterator, we can reserve space here
        let length = it.len();

        self.types.reserve(length);
        self.loc_symbols.reserve(length);

        for (symbol, loc_type) in it {
            let Loc { region, value } = loc_type;

            self.types.push(value);
            self.loc_symbols.push((symbol, region));
        }

        DefTypes {
            types: Slice::new(types_start as _, length as _),
            loc_symbols: Slice::new(loc_symbols_start as _, length as _),
        }
    }

    #[inline(always)]
    pub fn exists<I>(&mut self, flex_vars: I, defs_constraint: Constraint) -> Constraint
    where
        I: IntoIterator<Item = Variable>,
    {
        let defs_and_ret_constraint = Index::new(self.constraints.len() as _);

        self.constraints.push(defs_constraint);
        self.constraints.push(Constraint::True);

        let let_contraint = LetConstraint {
            rigid_vars: Slice::default(),
            flex_vars: self.variable_slice(flex_vars),
            def_types: DefTypes::default(),
            defs_and_ret_constraint,
        };

        let let_index = Index::new(self.let_constraints.len() as _);
        self.let_constraints.push(let_contraint);

        Constraint::Let(let_index, Slice::default())
    }

    #[inline(always)]
    pub fn exists_many<I, C>(&mut self, flex_vars: I, defs_constraint: C) -> Constraint
    where
        I: IntoIterator<Item = Variable>,
        C: IntoIterator<Item = Constraint>,
        C::IntoIter: ExactSizeIterator,
    {
        let defs_constraint = self.and_constraint(defs_constraint);

        let defs_and_ret_constraint = Index::new(self.constraints.len() as _);
        self.constraints.push(defs_constraint);
        self.constraints.push(Constraint::True);

        let let_contraint = LetConstraint {
            rigid_vars: Slice::default(),
            flex_vars: self.variable_slice(flex_vars),
            def_types: DefTypes::default(),
            defs_and_ret_constraint,
        };

        let let_index = Index::new(self.let_constraints.len() as _);
        self.let_constraints.push(let_contraint);

        Constraint::Let(let_index, Slice::default())
    }

    #[inline(always)]
    pub fn let_constraint<I1, I2, I3>(
        &mut self,
        rigid_vars: I1,
        flex_vars: I2,
        def_types: I3,
        defs_constraint: Constraint,
        ret_constraint: Constraint,
    ) -> Constraint
    where
        I1: IntoIterator<Item = Variable>,
        I2: IntoIterator<Item = Variable>,
        I3: IntoIterator<Item = (Symbol, Loc<Type>)>,
        I3::IntoIter: ExactSizeIterator,
    {
        // defs and ret constraint are stored consequtively, so we only need to store one index
        let defs_and_ret_constraint = Index::new(self.constraints.len() as _);

        self.constraints.push(defs_constraint);
        self.constraints.push(ret_constraint);

        let let_contraint = LetConstraint {
            rigid_vars: self.variable_slice(rigid_vars),
            flex_vars: self.variable_slice(flex_vars),
            def_types: self.def_types_slice(def_types),
            defs_and_ret_constraint,
        };

        let let_index = Index::new(self.let_constraints.len() as _);
        self.let_constraints.push(let_contraint);

        Constraint::Let(let_index, Slice::default())
    }

    /// A variant of `Let` used specifically for imports. When importing types from another module,
    /// we use a StorageSubs to store the data, and copy over the relevant
    /// variables/content/flattype/tagname etc.
    ///
    /// The general idea is to let-generalize the imorted types in the target module.
    /// More concretely, we need to simulate what `type_to_var` (solve.rs) does to a `Type`.
    /// While the copying puts all the data the right place, it misses that `type_to_var` puts
    /// the variables that it creates (to store the nodes of a Type in Subs) in the pool of the
    /// current rank (so they can be generalized).
    ///
    /// So, during copying of an import (`copy_import_to`, subs.rs) we track the variables that
    /// we need to put into the pool (simulating what `type_to_var` would do). Those variables
    /// then need to find their way to the pool, and a convenient approach turned out to be to
    /// tag them onto the `Let` that we used to add the imported values.
    #[inline(always)]
    pub fn let_import_constraint<I1, I2>(
        &mut self,
        rigid_vars: I1,
        def_types: I2,
        module_constraint: Constraint,
        pool_variables: &[Variable],
    ) -> Constraint
    where
        I1: IntoIterator<Item = Variable>,
        I2: IntoIterator<Item = (Symbol, Loc<Type>)>,
        I2::IntoIter: ExactSizeIterator,
    {
        // defs and ret constraint are stored consequtively, so we only need to store one index
        let defs_and_ret_constraint = Index::new(self.constraints.len() as _);

        self.constraints.push(Constraint::True);
        self.constraints.push(module_constraint);

        let let_contraint = LetConstraint {
            rigid_vars: self.variable_slice(rigid_vars),
            flex_vars: Slice::default(),
            def_types: self.def_types_slice(def_types),
            defs_and_ret_constraint,
        };

        let let_index = Index::new(self.let_constraints.len() as _);
        self.let_constraints.push(let_contraint);

        let pool_slice = self.variable_slice(pool_variables.iter().copied());

        Constraint::Let(let_index, pool_slice)
    }

    #[inline(always)]
    pub fn and_constraint<I>(&mut self, constraints: I) -> Constraint
    where
        I: IntoIterator<Item = Constraint>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut it = constraints.into_iter();

        match it.len() {
            0 => Constraint::True,
            1 => it.next().unwrap(),
            _ => {
                let start = self.constraints.len() as u32;

                self.constraints.extend(it);

                let end = self.constraints.len() as u32;

                let slice = Slice::new(start, (end - start) as u16);

                Constraint::And(slice)
            }
        }
    }

    pub fn lookup(
        &mut self,
        symbol: Symbol,
        expected: Expected<Type>,
        region: Region,
    ) -> Constraint {
        Constraint::Lookup(
            symbol,
            Index::push_new(&mut self.expectations, expected),
            region,
        )
    }

    pub fn contains_save_the_environment(&self, constraint: &Constraint) -> bool {
        match constraint {
            Constraint::SaveTheEnvironment => true,
            Constraint::Let(index, _) => {
                let let_constraint = &self.let_constraints[index.index()];

                let offset = let_constraint.defs_and_ret_constraint.index();
                let defs_constraint = &self.constraints[offset];
                let ret_constraint = &self.constraints[offset + 1];

                self.contains_save_the_environment(defs_constraint)
                    || self.contains_save_the_environment(ret_constraint)
            }
            Constraint::And(slice) => {
                let constraints = &self.constraints[slice.indices()];

                constraints
                    .iter()
                    .any(|c| self.contains_save_the_environment(c))
            }
            Constraint::Eq(..)
            | Constraint::Store(..)
            | Constraint::Lookup(..)
            | Constraint::Pattern(..)
            | Constraint::True
            | Constraint::IsOpenType(_)
            | Constraint::IncludesTag(_)
            | Constraint::PatternPresence(_, _, _, _)
            | Constraint::Exhaustive { .. }
            | Constraint::Resolve(..)
            | Constraint::CheckCycle(..) => false,
        }
    }

    pub fn store(
        &mut self,
        typ: Type,
        variable: Variable,
        filename: &'static str,
        line_number: u32,
    ) -> Constraint {
        let type_index = self.push_type(typ);
        let string_index = Index::push_new(&mut self.strings, filename);

        Constraint::Store(type_index, variable, string_index, line_number)
    }

    pub fn store_index(
        &mut self,
        type_index: EitherIndex<Type, Variable>,
        variable: Variable,
        filename: &'static str,
        line_number: u32,
    ) -> Constraint {
        let string_index = Index::push_new(&mut self.strings, filename);

        Constraint::Store(type_index, variable, string_index, line_number)
    }

    pub fn exhaustive(
        &mut self,
        real_var: Variable,
        real_region: Region,
        category_and_expectation: Result<
            (Category, Expected<Type>),
            (PatternCategory, PExpected<Type>),
        >,
        sketched_rows: SketchedRows,
        context: ExhaustiveContext,
        exhaustive: ExhaustiveMark,
    ) -> Constraint {
        let real_var = Self::push_type_variable(real_var);
        let sketched_rows = Index::push_new(&mut self.sketched_rows, sketched_rows);

        let equality = match category_and_expectation {
            Ok((category, expected)) => {
                let category = Index::push_new(&mut self.categories, category);
                let expected = Index::push_new(&mut self.expectations, expected);
                let equality = Eq(real_var, expected, category, real_region);
                let equality = Index::push_new(&mut self.eq, equality);
                Ok(equality)
            }
            Err((category, expected)) => {
                let category = Index::push_new(&mut self.pattern_categories, category);
                let expected = Index::push_new(&mut self.pattern_expectations, expected);
                let equality = PatternEq(real_var, expected, category, real_region);
                let equality = Index::push_new(&mut self.pattern_eq, equality);
                Err(equality)
            }
        };

        Constraint::Exhaustive(equality, sketched_rows, context, exhaustive)
    }

    pub fn check_cycle<I, I1>(
        &mut self,
        loc_symbols: I,
        expr_regions: I1,
        cycle_mark: IllegalCycleMark,
    ) -> Constraint
    where
        I: IntoIterator<Item = (Symbol, Region)>,
        I1: IntoIterator<Item = Region>,
    {
        let def_names = Slice::extend_new(&mut self.loc_symbols, loc_symbols);

        // we add a dummy symbol to these regions, so we can store the data in the loc_symbols vec
        let it = expr_regions.into_iter().map(|r| (Symbol::ATTR_ATTR, r));
        let expr_regions = Slice::extend_new(&mut self.loc_symbols, it);
        let expr_regions = Slice::new(expr_regions.start() as _, expr_regions.len() as _);

        let cycle = Cycle {
            def_names,
            expr_regions,
        };
        let cycle_index = Index::push_new(&mut self.cycles, cycle);

        Constraint::CheckCycle(cycle_index, cycle_mark)
    }
}

roc_error_macros::assert_sizeof_default!(Constraint, 3 * 8);
roc_error_macros::assert_sizeof_aarch64!(Constraint, 3 * 8);

#[derive(Clone, Copy, Debug)]
pub struct Eq(
    pub EitherIndex<Type, Variable>,
    pub Index<Expected<Type>>,
    pub Index<Category>,
    pub Region,
);

#[derive(Clone, Copy, Debug)]
pub struct PatternEq(
    pub EitherIndex<Type, Variable>,
    pub Index<PExpected<Type>>,
    pub Index<PatternCategory>,
    pub Region,
);

/// When we come across a lookup of an ability member, we'd like to try to specialize that
/// lookup during solving (knowing the specialization statically avoids re-solving during mono,
/// and always gives us a way to show what specialization was intended in the editor).
///
/// However, we attempting to resolve the specialization right at the lookup site is futile
/// (we may not have solved enough of the surrounding context to know the specialization).
/// So, we only collect what resolutions we'd like to make, and attempt to resolve them once
/// we pass through a let-binding (a def, or a normal `=` binding). At those positions, the
/// expression is generalized, so if there is a static specialization, we'd know it at that
/// point.
///
/// Note that this entirely opportunistic; if a lookup of an ability member uses it
/// polymorphically, we won't find its specialization(s) until monomorphization.
#[derive(Clone, Copy, Debug)]
pub struct OpportunisticResolve {
    /// The specialized type of this lookup, to try to resolve.
    pub specialization_variable: Variable,
    pub specialization_expectation: Index<Expected<Type>>,

    /// The ability member to try to resolve.
    pub member: Symbol,
    /// If we resolve a specialization, what specialization ID to store it on.
    pub specialization_id: SpecializationId,
}

#[derive(Clone, Copy)]
pub enum Constraint {
    Eq(Eq),
    Store(
        EitherIndex<Type, Variable>,
        Variable,
        Index<&'static str>,
        u32,
    ),
    Lookup(Symbol, Index<Expected<Type>>, Region),
    Pattern(
        EitherIndex<Type, Variable>,
        Index<PExpected<Type>>,
        Index<PatternCategory>,
        Region,
    ),
    /// Used for things that always unify, e.g. blanks and runtime errors
    True,
    SaveTheEnvironment,
    /// A Let constraint introduces symbols and their annotation at a certain level of nesting
    ///
    /// The `Slice<Variable>` is used for imports where we manually put the Content into Subs
    /// by copying from another module, but have to make sure that any variables we use to store
    /// these contents are added to `Pool` at the correct rank
    Let(Index<LetConstraint>, Slice<Variable>),
    And(Slice<Constraint>),
    /// Presence constraints
    IsOpenType(EitherIndex<Type, Variable>), // Theory; always applied to a variable? if yes the use that
    IncludesTag(Index<IncludesTag>),
    PatternPresence(
        EitherIndex<Type, Variable>,
        Index<PExpected<Type>>,
        Index<PatternCategory>,
        Region,
    ),
    Exhaustive(
        Result<Index<Eq>, Index<PatternEq>>,
        Index<SketchedRows>,
        ExhaustiveContext,
        ExhaustiveMark,
    ),
    /// Attempt to resolve a specialization.
    Resolve(OpportunisticResolve),
    CheckCycle(Index<Cycle>, IllegalCycleMark),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefTypes {
    pub types: Slice<Type>,
    pub loc_symbols: Slice<(Symbol, Region)>,
}

#[derive(Debug, Clone)]
pub struct LetConstraint {
    pub rigid_vars: Slice<Variable>,
    pub flex_vars: Slice<Variable>,
    pub def_types: DefTypes,
    pub defs_and_ret_constraint: Index<(Constraint, Constraint)>,
}

#[derive(Debug, Clone)]
pub struct IncludesTag {
    pub type_index: Index<Type>,
    pub tag_name: TagName,
    pub types: Slice<Type>,
    pub pattern_category: Index<PatternCategory>,
    pub region: Region,
}

#[derive(Debug, Clone, Copy)]
pub struct Cycle {
    pub def_names: Slice<(Symbol, Region)>,
    pub expr_regions: Slice<Region>,
}

/// Custom impl to limit vertical space used by the debug output
impl std::fmt::Debug for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq(Eq(arg0, arg1, arg2, arg3)) => {
                write!(f, "Eq({:?}, {:?}, {:?}, {:?})", arg0, arg1, arg2, arg3)
            }
            Self::Store(arg0, arg1, arg2, arg3) => {
                write!(f, "Store({:?}, {:?}, {:?}, {:?})", arg0, arg1, arg2, arg3)
            }
            Self::Lookup(arg0, arg1, arg2) => {
                write!(f, "Lookup({:?}, {:?}, {:?})", arg0, arg1, arg2)
            }
            Self::Pattern(arg0, arg1, arg2, arg3) => {
                write!(f, "Pattern({:?}, {:?}, {:?}, {:?})", arg0, arg1, arg2, arg3)
            }
            Self::True => write!(f, "True"),
            Self::SaveTheEnvironment => write!(f, "SaveTheEnvironment"),
            Self::Let(arg0, arg1) => f.debug_tuple("Let").field(arg0).field(arg1).finish(),
            Self::And(arg0) => f.debug_tuple("And").field(arg0).finish(),
            Self::IsOpenType(arg0) => f.debug_tuple("IsOpenType").field(arg0).finish(),
            Self::IncludesTag(arg0) => f.debug_tuple("IncludesTag").field(arg0).finish(),
            Self::PatternPresence(arg0, arg1, arg2, arg3) => {
                write!(
                    f,
                    "PatternPresence({:?}, {:?}, {:?}, {:?})",
                    arg0, arg1, arg2, arg3
                )
            }
            Self::Exhaustive(arg0, arg1, arg2, arg3) => {
                write!(
                    f,
                    "Exhaustive({:?}, {:?}, {:?}, {:?})",
                    arg0, arg1, arg2, arg3
                )
            }
            Self::Resolve(arg0) => {
                write!(f, "Resolve({:?})", arg0)
            }
            Self::CheckCycle(arg0, arg1) => {
                write!(f, "CheckCycle({:?}, {:?})", arg0, arg1)
            }
        }
    }
}
