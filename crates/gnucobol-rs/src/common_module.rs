//! Port of the **MODULE-lifecycle + runtime-settings surface** of `libcob/common.c` -- the
//! `cob_module_*` call-stack machinery, the `cob_save_func`/`cob_restore_func` argument frame
//! save/restore, the `cob_check_version` / `cob_parameter_check` CALL-time guards, the global/settings
//! accessors, and the `cob_set_runtime_option`/`cob_get_runtime_option` option switch.
//!
//! In libcob this state is global mutable storage: the `COB_MODULE_PTR` linked list of active modules
//! (threaded through `cob_module->next`), the `cobglobptr` global runtime block, the `cobsetptr`
//! settings block, and the `cob_module_list` registry of every activated module. Following the house
//! style (see `common.rs` / `common_exception.rs`) the global state is modelled here **explicitly** as
//! the [`ModuleRuntime`] struct -- the module call-stack is an explicit `Vec<CobModule>`, the global
//! and settings blocks are the [`CobGlobal`] / [`RuntimeSettings`] structs -- and passed by `&mut`.
//! Every function is the faithful 1:1 of its `common.c` original. No global statics, no `unsafe`,
//! panic-free.
//!
//! The genuinely OS-bound leaves -- `cob_init` (signal handlers, env scan, locale), `cob_cache_malloc`
//! / `cob_free` (the allocator), `cob_runtime_error` / `cob_hard_failure` (the abort path),
//! `cob_rescan_env_vals`, and the `FILE *` file handles set via the runtime-option switch -- are the
//! declared runtime boundary (the same BDB/EXTFH boundary pattern used elsewhere in the port): the pure
//! state transition is ported and the OS leaf is modelled as a documented effect on the struct.

#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Local helper constants (redefined locally per the no-cross-module rule).
// ---------------------------------------------------------------------------

/// `config.h:PACKAGE_VERSION` -- the libcob package version string ("3.2" for GnuCOBOL 3.2).
pub const PACKAGE_VERSION: &str = "3.2";
/// `config.h:PATCH_LEVEL` -- the libcob patch level (0 for GnuCOBOL 3.2).
pub const PATCH_LEVEL: i32 = 0;

/// `common.c:MAX_MODULE_ITERS` -- the broken-chain guard limit walked by [`cob_module_global_enter`].
pub const MAX_MODULE_ITERS: i32 = 10240;

/// `enum cob_statement:STMT_UNKNOWN` -- the "no current statement" sentinel.
pub const STMT_UNKNOWN: u32 = 0;
/// `enum cob_statement:STMT_ENTRY` -- the ENTRY-point statement (used by `raise_arg_mismatch`).
pub const STMT_ENTRY: u32 = 1;

/// `exception.def` index for `COB_EC_PROGRAM_RECURSIVE_CALL` (the `EC-PROGRAM-RECURSIVE-CALL` id).
/// Redefined locally; the canonical table lives in `common_exception`.
pub const COB_EC_PROGRAM_RECURSIVE_CALL: usize = 93;
/// `exception.def` index for `COB_EC_PROGRAM_ARG_MISMATCH` (the `EC-PROGRAM-ARG-MISMATCH` id).
pub const COB_EC_PROGRAM_ARG_MISMATCH: usize = 87;

// ---------------------------------------------------------------------------
// cob_module  -- the per-module activation record.
// ---------------------------------------------------------------------------

/// Port of `common.h:typedef struct __cob_module` -- the per-module activation record pushed onto the
/// `COB_MODULE_PTR` call stack. Only the fields touched by the lifecycle/version/argument functions
/// ported here are modelled; the `cob_field *` / `cob_call_union` / XML/JSON pointer members are the
/// runtime data-pointer boundary and are out of scope for this pure-state port. The C `next` pointer is
/// represented implicitly by the [`ModuleRuntime`] stack ordering (top of `Vec` == `COB_MODULE_PTR`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CobModule {
    /// `module_name` -- the module's name (`const char *`).
    pub module_name: Option<String>,
    /// `module_sources` -- the compiled source-module names (`const char **`).
    pub module_sources: Vec<String>,
    /// `module_stmt` -- position of the last statement executed (modulated source line + source index).
    pub module_stmt: u32,
    /// `statement` -- the `enum cob_statement` currently executed.
    pub statement: u32,
    /// `module_num_params` -- the module's argument count (saved from `cob_call_params` on entry).
    pub module_num_params: i32,
    /// `module_active` -- non-zero while the module is active.
    pub module_active: u32,
    /// `gc_version` -- the module's GnuCOBOL version string (set by [`cob_check_version`] up to 3.1.2).
    pub gc_version: Option<String>,
    /// `ebcdic_sign` -- DISPLAY SIGN is EBCDIC.
    pub ebcdic_sign: u8,
    /// `flag_main` -- this is the main module.
    pub flag_main: u8,
    /// `flag_filename_mapping` -- filename mapping enabled.
    pub flag_filename_mapping: u8,
    /// `flag_binary_truncate` -- binary truncation enabled.
    pub flag_binary_truncate: u8,
    /// `flag_exit_program` -- exit after CALL.
    pub flag_exit_program: u8,
    /// `flag_did_cancel` -- the module has been cancelled.
    pub flag_did_cancel: u8,
    /// `cob_procedure_params` length -- modelled as the count of argument slots currently bound (the
    /// `cob_field **cob_procedure_params` array itself is the data-pointer boundary).
    pub cob_procedure_params_len: usize,
}

impl CobModule {
    /// A zeroed module record (the libcob `cob_cache_malloc (sizeof (cob_module))` allocates zeroed).
    pub fn new() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// cob_global  -- the runtime global block (the fields these functions touch).
// ---------------------------------------------------------------------------

/// Port of the `cob_global` runtime block (`cobglobptr`), restricted to the fields touched by the
/// module-lifecycle / version / parameter functions ported here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CobGlobal {
    /// `cob_call_params` -- the number of parameters the current CALL passed.
    pub cob_call_params: i32,
    /// `cob_stmt_exception` -- set when the active statement has an `ON EXCEPTION` phrase (so a
    /// recursive-call / arg-mismatch returns to the caller instead of aborting).
    pub cob_stmt_exception: i32,
}

// ---------------------------------------------------------------------------
// RuntimeSettings  -- cob_settings (cobsetptr), the runtime-option-switch surface.
// ---------------------------------------------------------------------------

/// An opaque `FILE *` handle modelled as a boundary token. libcob stores real `FILE *` here; the port
/// represents "a handle is set" as `Some(id)` and "NULL" as `None`. The `id` is an opaque caller token
/// (e.g. a file-descriptor or table index); the port never dereferences it.
pub type FileHandle = Option<u64>;

/// Port of the `cob_settings` block (`cobsetptr`), restricted to the runtime-option-switch surface read
/// / written by [`cob_set_runtime_option`] / [`cob_get_runtime_option`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeSettings {
    /// `cob_trace_file` -- the trace output `FILE *`.
    pub cob_trace_file: FileHandle,
    /// `external_trace_file` -- set when `cob_trace_file` was supplied by the caller (not libcob-owned).
    pub external_trace_file: i32,
    /// `cob_display_print_file` -- the DISPLAY UPON PRINTER `FILE *` (always external when set).
    pub cob_display_print_file: FileHandle,
    /// `cob_display_punch_file` -- the DISPLAY UPON SYSPUNCH `FILE *` (always external when set here).
    pub cob_display_punch_file: FileHandle,
    /// `cob_display_punch_filename` -- the libcob-owned punch filename, if libcob opened the file.
    pub cob_display_punch_filename: Option<String>,
    /// `cob_dump_file` -- the abort-dump `FILE *` (always external; libcob only opens it on abort).
    pub cob_dump_file: FileHandle,
    /// `cob_dump_filename` -- the dump filename ("NONE" when no external dump file is set).
    pub cob_dump_filename: Option<String>,
    /// Set by [`cob_set_runtime_option`] when the `RESCAN_ENV` option requests an env re-scan; libcob
    /// calls `cob_rescan_env_vals()` here -- the OS leaf is recorded as this counter.
    pub rescan_env_requested: u32,
}

/// Port of `common.h:enum cob_runtime_option_switch` -- the option selector for
/// [`cob_set_runtime_option`] / [`cob_get_runtime_option`]. The discriminant values match the C enum
/// exactly (so a faithful round-trip of the integer selector is possible).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CobRuntimeOptionSwitch {
    /// `COB_SET_RUNTIME_TRACE_FILE = 0` -- `p` is `FILE *`.
    TraceFile = 0,
    /// `COB_SET_RUNTIME_DISPLAY_PRINTER_FILE = 1` -- `p` is `FILE *`.
    DisplayPrinterFile = 1,
    /// `COB_SET_RUNTIME_RESCAN_ENV = 2` -- rescan environment variables.
    RescanEnv = 2,
    /// `COB_SET_RUNTIME_DISPLAY_PUNCH_FILE = 3` -- `p` is `FILE *`.
    DisplayPunchFile = 3,
    /// `COB_SET_RUNTIME_DUMP_FILE = 4` -- `p` is `FILE *`.
    DumpFile = 4,
}

impl CobRuntimeOptionSwitch {
    /// Map a raw C `enum` integer to the switch case (`None` for an unknown option, which the C `switch`
    /// routes to the `default:` runtime-warning branch).
    pub fn from_i32(opt: i32) -> Option<Self> {
        match opt {
            0 => Some(Self::TraceFile),
            1 => Some(Self::DisplayPrinterFile),
            2 => Some(Self::RescanEnv),
            3 => Some(Self::DisplayPunchFile),
            4 => Some(Self::DumpFile),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// version_bitstring / cob_check_version decision (PURE).
// ---------------------------------------------------------------------------

/// A parsed `major.minor.point` version triple (`common.c:struct ver_t`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VerT {
    /// `major` component.
    pub major: i32,
    /// `minor` component.
    pub minor: i32,
    /// `point` component.
    pub point: i32,
    /// `version` -- the packed bitstring (see [`version_bitstring`]).
    pub version: u32,
}

/// Port of `common.c:version_bitstring` -- pack `major<<24 | minor<<16 | point<<8` (the `point`-and-up
/// bits; the low byte is unused). Already ported elsewhere; redefined locally per the no-cross-module
/// rule because [`cob_check_version`]'s decision needs it.
pub fn version_bitstring(module: VerT) -> u32 {
    (((module.major as u32) & 0xff) << 24)
        | (((module.minor as u32) & 0xff) << 16)
        | (((module.point as u32) & 0xff) << 8)
}

/// Parse up to 3 dotted decimal components ("M", "M.m" or "M.m.p"), returning `(VerT, nparts)` where
/// `nparts` is how many components parsed -- the libcob `sscanf("%d.%d.%d", ...)` return. Non-numeric
/// or missing components leave the corresponding field at its default (0).
fn parse_ver(s: &str) -> (VerT, i32) {
    let mut v = VerT::default();
    let mut n = 0i32;
    let mut it = s.split('.');
    if let Some(part) = it.next() {
        if let Ok(x) = part.trim().parse::<i32>() {
            v.major = x;
            n += 1;
        } else {
            return (v, 0);
        }
    }
    if let Some(part) = it.next() {
        if let Ok(x) = part.trim().parse::<i32>() {
            v.minor = x;
            n += 1;
        } else {
            return (v, n);
        }
    }
    if let Some(part) = it.next() {
        if let Ok(x) = part.trim().parse::<i32>() {
            v.point = x;
            n += 1;
        }
    }
    (v, n)
}

/// The pure outcome of [`cob_check_version`]: whether the program's version is accepted, and (on
/// rejection) the warning text libcob would emit before `cob_hard_failure()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionCheck {
    /// The program version is compatible -- `cob_check_version` returns without warning.
    Ok {
        /// Whether `cannot_check_subscript` is set (the program is at or below 2.2, so the runtime
        /// cannot validate subscripts compiled by it).
        cannot_check_subscript: bool,
    },
    /// The program version is incompatible -- libcob emits these hints then hard-fails.
    Mismatch {
        /// `cob_runtime_error(_("version mismatch"))` text.
        error: String,
        /// First `cob_runtime_hint` -- the program's `prog has version packver.patchlev`.
        prog_hint: String,
        /// Second `cob_runtime_hint` -- libcob's own `libcob has version PACKAGE_VERSION.PATCH_LEVEL`.
        lib_hint: String,
    },
}

/// Port of `common.c:cob_check_version` -- compare a program's packed version (`packver_prog`,
/// `patchlev_prog`) against the libcob version (`PACKAGE_VERSION`, `PATCH_LEVEL`). The **decision** is
/// pure given the two versions and is returned as a [`VersionCheck`]; the C side then either returns
/// (Ok) or emits the runtime-error/hints and `cob_hard_failure()`s (Mismatch -- the abort path is the
/// boundary). When Ok, the caller is expected to stamp `COB_MODULE_PTR->gc_version` with `packver_prog`
/// if unset; see [`cob_check_version_apply`] for the stateful wrapper.
pub fn cob_check_version(prog: &str, packver_prog: &str, patchlev_prog: i32) -> VersionCheck {
    let mut lib = VerT {
        major: 9,
        minor: 9,
        point: 9,
        version: 0,
    };
    let (parsed_lib, nparts) = parse_ver(PACKAGE_VERSION);
    // sscanf overwrites only the components it parses; mirror that (lib starts {9,9,9}).
    if nparts >= 1 {
        lib.major = parsed_lib.major;
    }
    if nparts >= 2 {
        lib.minor = parsed_lib.minor;
    }
    if nparts >= 3 {
        lib.point = parsed_lib.point;
    }

    if nparts >= 2 {
        lib.version = version_bitstring(lib);

        let (mut app, _) = parse_ver(packver_prog);
        app.version = version_bitstring(app);

        if app.major == 2 && app.minor < 2 {
            return mismatch(prog, packver_prog, patchlev_prog);
        }

        if app.version == lib.version && patchlev_prog <= PATCH_LEVEL {
            return VersionCheck::Ok {
                cannot_check_subscript: false,
            };
        }
        if app.version < lib.version {
            // we only claim compatibility to 2.2+
            let mut minimal = VerT {
                major: 2,
                minor: 2,
                point: 0,
                version: 0,
            };
            let mut cannot_check_subscript = false;
            if app.version <= version_bitstring(minimal) {
                cannot_check_subscript = true;
            }
            minimal.minor = 1;
            if app.version >= version_bitstring(minimal) {
                return VersionCheck::Ok {
                    cannot_check_subscript,
                };
            }
        }
    }

    mismatch(prog, packver_prog, patchlev_prog)
}

/// Build the [`VersionCheck::Mismatch`] payload (the `err:` label of `cob_check_version`).
fn mismatch(prog: &str, packver_prog: &str, patchlev_prog: i32) -> VersionCheck {
    VersionCheck::Mismatch {
        error: "version mismatch".to_string(),
        prog_hint: format!("{} has version {}.{}", prog, packver_prog, patchlev_prog),
        lib_hint: format!("{} has version {}.{}", "libcob", PACKAGE_VERSION, PATCH_LEVEL),
    }
}

// ---------------------------------------------------------------------------
// cob_func_loc  -- the saved argument frame (cob_save_func / cob_restore_func).
// ---------------------------------------------------------------------------

/// Port of `common.c:struct cob_func_loc` -- the saved module environment returned by
/// [`ModuleRuntime::cob_save_func`] and consumed by [`ModuleRuntime::cob_restore_func`]. The C struct
/// also carries `func_params` / `data` (`void *` arrays of `cob_field *`); those are modelled as the
/// count of bound parameters, the rest being the data-pointer boundary.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CobFuncLoc {
    /// `save_call_params` -- the caller's `cobglobptr->cob_call_params`.
    pub save_call_params: i32,
    /// `save_num_params` -- the caller's `COB_MODULE_PTR->module_num_params`.
    pub save_num_params: i32,
    /// `save_proc_parms` length -- the caller's `cob_procedure_params` array length.
    pub save_proc_parms_len: usize,
    /// The number of parameter slots allocated for this frame (`numparams`).
    pub func_params_len: usize,
}

// ---------------------------------------------------------------------------
// ModuleRuntime  -- the explicit COB_MODULE_PTR call stack + global/settings blocks.
// ---------------------------------------------------------------------------

/// The explicit model of libcob's module/global/settings global state. `module_stack` is the
/// `COB_MODULE_PTR` linked list (the **last** element is `COB_MODULE_PTR`, the active module; pushing
/// is `enter`, popping is `leave`). `module_list` is the `cob_module_list` registry of every activated
/// module (modelled by stable id). `glob` / `settings` are `cobglobptr` / `cobsetptr`.
#[derive(Debug, Clone, Default)]
pub struct ModuleRuntime {
    /// The `COB_MODULE_PTR` call stack; `module_stack.last()` is the active module.
    pub module_stack: Vec<CobModule>,
    /// Parallel id stack kept in lock-step with `module_stack`: `module_id_stack[i]` is the
    /// persistent-slot id (pointer identity) of `module_stack[i]`. Lets recursion detection compare
    /// module identity without giving [`CobModule`] an explicit `next`/id field the C struct hides from
    /// these functions.
    module_id_stack: Vec<u64>,
    /// The `cob_module_list` registry -- stable ids of every module ever activated (and not freed).
    pub module_list: Vec<u64>,
    /// `cobglobptr` -- the runtime global block.
    pub glob: CobGlobal,
    /// `cobsetptr` -- the runtime settings block.
    pub settings: RuntimeSettings,
    /// `cob_initialized` -- whether `cob_init` has run.
    pub cob_initialized: bool,
    /// `cob_argc` -- the saved argument count (main-program parameter count source).
    pub cob_argc: i32,
    /// `check_mainhandle` -- cleared by [`Self::cob_init_nomain`].
    pub check_mainhandle: i32,
    /// `cannot_check_subscript` -- file-static flag set by [`cob_check_version`] for pre-2.2 modules.
    pub cannot_check_subscript: i32,
    /// `exit_code` -- the last `cob_call` result, set by [`Self::cob_call_with_exception_check`].
    pub exit_code: i32,
    /// A monotonically increasing id source for newly activated modules (models pointer identity).
    next_module_id: u64,
}

/// The result of an `enter`/`global_enter`: `Ok(0)` for a normal entry, `Ok(1)` for "recursive CALL
/// with ON EXCEPTION -> return to caller", or `Err` for the libcob fatal/abort leaf (the boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnterError {
    /// `cob_fatal_error(COB_FERROR_INITIALIZED)` -- runtime not initialized and `auto_init` is false.
    NotInitialized,
    /// `cob_fatal_error(COB_FERROR_RECURSIVE)` -- a non-exception recursive call was detected.
    Recursive,
}

impl ModuleRuntime {
    /// A fresh, uninitialized runtime (no modules, `cob_initialized == false`, `check_mainhandle == 1`).
    pub fn new() -> Self {
        Self {
            check_mainhandle: 1,
            ..Self::default()
        }
    }

    /// Port of `common.c:cob_is_initialized` -- whether `cobglobptr != NULL` (i.e. `cob_init` has run).
    pub fn cob_is_initialized(&self) -> bool {
        self.cob_initialized
    }

    /// Port of `common.c:cob_get_global_ptr` -- return the runtime global block, fataling if the runtime
    /// is not initialized. The fatal leaf (`cob_fatal_error(COB_FERROR_INITIALIZED)`) is the boundary,
    /// modelled as `Err(EnterError::NotInitialized)`.
    pub fn cob_get_global_ptr(&self) -> Result<&CobGlobal, EnterError> {
        if !self.cob_initialized {
            return Err(EnterError::NotInitialized);
        }
        Ok(&self.glob)
    }

    /// Port of `common.c:cob_get_settings_ptr` -- return the settings block (`cobsetptr`).
    pub fn cob_get_settings_ptr(&self) -> &RuntimeSettings {
        &self.settings
    }

    /// Mutable settings accessor (the C side returns the raw `cobsetptr` for in-place mutation).
    pub fn cob_get_settings_ptr_mut(&mut self) -> &mut RuntimeSettings {
        &mut self.settings
    }

    /// Port of `common.c:cob_module_global_enter` -- push a module onto the `COB_MODULE_PTR` stack.
    ///
    /// `module` is the program's persistent module slot (`Some(id)` once allocated, `None` on first
    /// entry). On first entry a fresh [`CobModule`] is allocated, registered in `module_list`, and its
    /// id returned. On re-entry with `entry == 0` the stack is walked for the same module id: a
    /// recursive call either returns `Ok(1)` (when `cob_stmt_exception` is set: raises
    /// `EC-PROGRAM-RECURSIVE-CALL` and returns to caller) or hits the recursive fatal boundary
    /// (`Err(Recursive)`). The broken-chain guard ([`MAX_MODULE_ITERS`]) bounds the walk. Returns the
    /// module's id alongside the C return code.
    ///
    /// `set_exception` is the caller's `cob_set_exception` hook (the exception-state machinery lives in
    /// `common_exception`); it is invoked with [`COB_EC_PROGRAM_RECURSIVE_CALL`] on the ON-EXCEPTION
    /// path.
    pub fn cob_module_global_enter(
        &mut self,
        module: &mut Option<u64>,
        auto_init: bool,
        entry: i32,
        mut set_exception: impl FnMut(usize),
    ) -> Result<(i32, u64), EnterError> {
        // Check initialized.
        if !self.cob_initialized {
            if auto_init {
                self.cob_init(0, &[]);
            } else {
                return Err(EnterError::NotInitialized);
            }
        }

        // Check module pointer.
        let module_id = match *module {
            None => {
                // Allocate a fresh module and add to list of all modules activated.
                let id = self.alloc_module();
                *module = Some(id);
                self.module_list.insert(0, id);
                self.push_new_module(id);
                id
            }
            Some(id) => {
                if entry == 0 {
                    // Walk COB_MODULE_PTR for the same module (recursion detection).
                    let mut k = 0;
                    // iterate from top of stack (COB_MODULE_PTR) downward via stored ids
                    for &mod_id in self.module_id_chain().iter() {
                        if id == mod_id {
                            if self.glob.cob_stmt_exception != 0 {
                                // CALL has ON EXCEPTION so return to caller.
                                set_exception(COB_EC_PROGRAM_RECURSIVE_CALL);
                                self.glob.cob_stmt_exception = 0;
                                return Ok((1, id));
                            }
                            return Err(EnterError::Recursive);
                        }
                        k += 1;
                        if k == MAX_MODULE_ITERS {
                            // prevent endless loop in case of broken list
                            break;
                        }
                    }
                }
                self.push_existing_module(id);
                id
            }
        };

        // Save parameter count, get number from argc if main program.
        if self.module_stack.len() == 1 {
            // !COB_MODULE_PTR before push == empty stack == main program; after push len()==1.
            self.glob.cob_call_params = self.cob_argc - 1;
        }
        if let Some(top) = self.module_stack.last_mut() {
            top.module_num_params = self.glob.cob_call_params;
            top.module_stmt = 0;
            top.statement = STMT_UNKNOWN;
        }
        self.glob.cob_stmt_exception = 0;
        Ok((0, module_id))
    }

    /// Port of `common.c:cob_module_enter` -- the void wrapper over [`Self::cob_module_global_enter`]
    /// with `entry == 0` and a no-op name-hash (`cob_module_global_enter(module, mglobal, auto_init,
    /// 0, 0)`); the return code is discarded.
    pub fn cob_module_enter(
        &mut self,
        module: &mut Option<u64>,
        auto_init: bool,
        set_exception: impl FnMut(usize),
    ) -> Result<(), EnterError> {
        self.cob_module_global_enter(module, auto_init, 0, set_exception)
            .map(|_| ())
    }

    /// Port of `common.c:cob_module_leave` -- pop `COB_MODULE_PTR` (`COB_MODULE_PTR = COB_MODULE_PTR->next`).
    /// The `module` argument is `COB_UNUSED` in the C source. Returns the popped module, if any.
    pub fn cob_module_leave(&mut self) -> Option<CobModule> {
        self.module_id_stack.pop();
        self.module_stack.pop()
    }

    /// Port of `common.c:cob_module_free` -- remove a module (by id) from `cob_module_list` and free its
    /// storage (`cob_cache_free`, the allocator boundary), setting the caller's slot to `NULL`. A `None`
    /// slot is a no-op (the `if (*module == NULL) return;` guard).
    pub fn cob_module_free(&mut self, module: &mut Option<u64>) {
        let id = match *module {
            None => return,
            Some(id) => id,
        };
        // Remove from list of all modules activated.
        if let Some(pos) = self.module_list.iter().position(|&m| m == id) {
            self.module_list.remove(pos);
        }
        // cob_cache_free(*module) -- the allocator leaf; here drop any stack copy with this id is N/A
        // (cob_module_free is for the persistent slot, not the active stack). *module = NULL.
        *module = None;
    }

    /// Port of `common.c:cob_save_func` -- save the current module's argument frame and bind a new one
    /// of `numparams = min(params, eparams)` slots. The varargs `cob_field *` list is the data-pointer
    /// boundary, modelled as the slot count `numparams`. Returns the [`CobFuncLoc`] to be restored by
    /// [`Self::cob_restore_func`].
    pub fn cob_save_func(&mut self, params: i32, eparams: i32) -> CobFuncLoc {
        let numparams = if params > eparams { eparams } else { params };
        let numparams_usize = numparams.max(0) as usize;

        let (save_call_params, save_num_params, save_proc_parms_len) =
            if let Some(top) = self.module_stack.last() {
                (
                    self.glob.cob_call_params,
                    top.module_num_params,
                    top.cob_procedure_params_len,
                )
            } else {
                (self.glob.cob_call_params, 0, 0)
            };

        let fl = CobFuncLoc {
            save_call_params,
            save_num_params,
            save_proc_parms_len,
            func_params_len: numparams_usize,
        };

        // Set current values.
        if let Some(top) = self.module_stack.last_mut() {
            top.cob_procedure_params_len = numparams_usize;
        }
        self.glob.cob_call_params = numparams;
        fl
    }

    /// Port of `common.c:cob_restore_func` -- restore the calling environment saved by
    /// [`Self::cob_save_func`] and free the frame (`cob_free`, the allocator boundary).
    pub fn cob_restore_func(&mut self, fl: &CobFuncLoc) {
        self.glob.cob_call_params = fl.save_call_params;
        if let Some(top) = self.module_stack.last_mut() {
            top.cob_procedure_params_len = fl.save_proc_parms_len;
            top.module_num_params = fl.save_num_params;
        }
        // cob_free(fl->data); cob_free(fl->func_params); cob_free(fl); -- allocator boundary.
    }

    /// Port of `common.c:cob_check_version` (stateful wrapper) -- run the pure [`cob_check_version`]
    /// decision, then apply its side effects: stamp `COB_MODULE_PTR->gc_version` with `packver_prog`
    /// when the runtime/module is present and unset, set `cannot_check_subscript` on an Ok pre-2.2
    /// program. Returns the [`VersionCheck`]; a `Mismatch` is the hard-failure boundary for the caller.
    pub fn cob_check_version_apply(
        &mut self,
        prog: &str,
        packver_prog: &str,
        patchlev_prog: i32,
    ) -> VersionCheck {
        // Stamp gc_version first (the C side does this before the accept/reject return, when nparts>=2).
        if let Some(top) = self.module_stack.last_mut() {
            if top.gc_version.is_none() {
                top.gc_version = Some(packver_prog.to_string());
            }
        }
        let res = cob_check_version(prog, packver_prog, patchlev_prog);
        if let VersionCheck::Ok {
            cannot_check_subscript: true,
        } = res
        {
            self.cannot_check_subscript = 1;
        }
        res
    }

    /// Port of `common.c:cob_parameter_check` -- verify a CALL passed at least `num_arguments`
    /// parameters. Returns `Ok(())` when satisfied, or `Err(diagnostic)` carrying the
    /// `cob_runtime_error` text -- the C side then `cob_hard_failure()`s (the abort boundary).
    pub fn cob_parameter_check(
        &self,
        func_name: &str,
        num_arguments: i32,
    ) -> Result<(), String> {
        if self.glob.cob_call_params < num_arguments {
            return Err(format!(
                "CALL to {} requires {} arguments",
                func_name, num_arguments
            ));
        }
        Ok(())
    }

    /// Port of `common.c:cob_set_runtime_option` -- the `switch (opt)` over the runtime-option selector.
    /// `p` is the option's `void *` payload, modelled per-case ([`FileHandle`] for the file options,
    /// ignored for `RESCAN_ENV`). The `FILE *` close / `cob_free` / `cob_rescan_env_vals()` leaves are
    /// the OS boundary (modelled as struct mutation + the `rescan_env_requested` counter). An unknown
    /// option returns the `default:` runtime-warning text instead of fataling.
    pub fn cob_set_runtime_option(
        &mut self,
        opt: CobRuntimeOptionSwitch,
        p: FileHandle,
    ) -> Result<(), String> {
        match opt {
            CobRuntimeOptionSwitch::TraceFile => {
                self.settings.cob_trace_file = p;
                self.settings.external_trace_file = if p.is_some() { 1 } else { 0 };
            }
            CobRuntimeOptionSwitch::DisplayPrinterFile => {
                // note: if set cob_display_print_file is always external
                self.settings.cob_display_print_file = p;
            }
            CobRuntimeOptionSwitch::DisplayPunchFile => {
                // note: if set cob_display_punch_file is always external
                if self.settings.cob_display_punch_filename.is_some() {
                    // if previously opened by libcob: close and free pointer to filename (OS leaf).
                    self.settings.cob_display_punch_filename = None;
                }
                self.settings.cob_display_punch_file = p;
            }
            CobRuntimeOptionSwitch::DumpFile => {
                // note: if set cob_dump_file is always external.
                self.settings.cob_dump_file = p;
                if self.settings.cob_dump_file.is_none() {
                    // cob_free old filename, set "NONE".
                    self.settings.cob_dump_filename = Some("NONE".to_string());
                }
            }
            CobRuntimeOptionSwitch::RescanEnv => {
                // cob_rescan_env_vals() -- the OS leaf, recorded as a request.
                self.settings.rescan_env_requested += 1;
            }
        }
        Ok(())
    }

    /// Port of `common.c:cob_set_runtime_option` driven by a raw C `enum` integer (the undocumented-ABI
    /// entry). An unknown option returns `Err` with the `default:` runtime-warning text.
    pub fn cob_set_runtime_option_raw(&mut self, opt: i32, p: FileHandle) -> Result<(), String> {
        match CobRuntimeOptionSwitch::from_i32(opt) {
            Some(o) => self.cob_set_runtime_option(o, p),
            None => Err(format!(
                "{} called with unknown option: {}",
                "cob_set_runtime_option", opt
            )),
        }
    }

    /// Port of `common.c:cob_get_runtime_option` -- return the current `FILE *` for the given option.
    /// `DISPLAY_PUNCH_FILE` is only externalized when not acquired by libcob (i.e. no libcob-owned
    /// filename), matching the C guard. `RESCAN_ENV` is not a gettable option (the C `default:` path
    /// is the unknown-option [`Err`] runtime-error).
    pub fn cob_get_runtime_option(
        &self,
        opt: CobRuntimeOptionSwitch,
    ) -> Result<FileHandle, String> {
        match opt {
            CobRuntimeOptionSwitch::TraceFile => Ok(self.settings.cob_trace_file),
            CobRuntimeOptionSwitch::DisplayPrinterFile => Ok(self.settings.cob_display_print_file),
            CobRuntimeOptionSwitch::DisplayPunchFile => {
                // only externalize if not aquired by libcob
                if self.settings.cob_display_punch_filename.is_some() {
                    Ok(None)
                } else {
                    Ok(self.settings.cob_display_punch_file)
                }
            }
            CobRuntimeOptionSwitch::DumpFile => Ok(self.settings.cob_dump_file),
            CobRuntimeOptionSwitch::RescanEnv => Err(format!(
                "{} called with unknown option: {}",
                "cob_get_runtime_option",
                CobRuntimeOptionSwitch::RescanEnv as i32
            )),
        }
    }

    /// Port of `common.c:cob_init_nomain` -- clear `check_mainhandle`, then `cob_init`. The `argc`/`argv`
    /// are passed through to [`Self::cob_init`].
    pub fn cob_init_nomain(&mut self, argc: i32, argv: &[String]) {
        self.check_mainhandle = 0;
        self.cob_init(argc, argv);
    }

    /// Port of `common.c:cob_extern_init` -- initialize the runtime if not already (`if (!cob_initialized)
    /// cob_init(0, NULL);`); idempotent. Returns `0` (the C return). The init proper is the boundary
    /// ([`Self::cob_init`]).
    pub fn cob_extern_init(&mut self) -> i32 {
        if !self.cob_initialized {
            self.cob_init(0, &[]);
        }
        0
    }

    /// Port of `common.c:cob_init` -- the genuinely OS-bound runtime initialization (signal handlers,
    /// locale/NLS, environment scan, allocator base, global/settings allocation). The OS leaves are the
    /// declared boundary; the **pure** state transition modelled here is: the once-only guard (`if
    /// (cob_initialized) return;`), recording `cob_argc`, and flipping `cob_initialized` (which makes
    /// `cobglobptr`/`cobsetptr` "non-NULL"). The full env/signal/locale work is out of scope.
    pub fn cob_init(&mut self, argc: i32, _argv: &[String]) {
        // Ensure initialization is only done once.
        if self.cob_initialized {
            return;
        }
        // cob_set_signal(); allocator base reset; global/settings allocation -- OS boundary.
        self.cob_argc = argc;
        self.cob_initialized = true;
    }

    /// Port of `common.c:cob_call_with_exception_check` -- the pure dispatch part: call `cob_call(name,
    /// argc, argv)` and store the result in `exit_code`, returning `0` for a normal return. The
    /// `setjmp`/`longjmp` non-local return (`return_jmp_buf`, used to catch exit/error/signal) is the
    /// OS/control-flow boundary: a `longjmp` back is modelled as the caller supplying a non-zero
    /// `jmp_result`, in which case the function returns that code without calling. `call` is the
    /// `cob_call` dispatch hook (resolves and invokes the named program).
    pub fn cob_call_with_exception_check(
        &mut self,
        name: &str,
        argc: i32,
        argv_len: usize,
        jmp_result: i32,
        mut call: impl FnMut(&str, i32, usize) -> i32,
    ) -> i32 {
        // ret = setjmp(return_jmp_buf); if (ret) { return_jmp_buffer_set = 0; return ret; }
        if jmp_result != 0 {
            return jmp_result;
        }
        self.exit_code = call(name, argc, argv_len);
        0
    }

    // ---- internal stack helpers (model COB_MODULE_PTR pointer identity) ----

    /// Allocate a fresh module id (models `cob_cache_malloc(sizeof(cob_module))` pointer identity).
    fn alloc_module(&mut self) -> u64 {
        self.next_module_id += 1;
        self.next_module_id
    }

    /// Push a brand-new zeroed module (with the given id) onto the stack.
    fn push_new_module(&mut self, id: u64) {
        let mut m = CobModule::new();
        m.module_active = 1;
        self.module_id_stack.push(id);
        self.module_stack.push(m);
    }

    /// Push an existing (persistent-slot) module id onto the stack -- a fresh activation record bearing
    /// the same id (the C side reuses the same `cob_module` storage; here a parallel record is pushed).
    fn push_existing_module(&mut self, id: u64) {
        let mut m = CobModule::new();
        m.module_active = 1;
        self.module_id_stack.push(id);
        self.module_stack.push(m);
    }

    /// The ids of the modules currently on the stack, top (`COB_MODULE_PTR`) first.
    fn module_id_chain(&self) -> Vec<u64> {
        let mut v = self.module_id_stack.clone();
        v.reverse();
        v
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn init_rt() -> ModuleRuntime {
        let mut rt = ModuleRuntime::new();
        rt.cob_init(1, &[]);
        rt
    }

    #[test]
    fn version_bitstring_packs_major_minor_point() {
        let v = VerT {
            major: 3,
            minor: 2,
            point: 0,
            version: 0,
        };
        assert_eq!(version_bitstring(v), 0x03_02_00_00);
    }

    #[test]
    fn parse_ver_three_parts() {
        let (v, n) = parse_ver("3.2.0");
        assert_eq!(n, 3);
        assert_eq!((v.major, v.minor, v.point), (3, 2, 0));
    }

    #[test]
    fn check_version_matching_is_ok() {
        let res = cob_check_version("prog", "3.2", 0);
        assert_eq!(
            res,
            VersionCheck::Ok {
                cannot_check_subscript: false
            }
        );
    }

    #[test]
    fn check_version_pre_2_2_mismatches() {
        // app.major == 2 && app.minor < 2 -> mismatch.
        match cob_check_version("prog", "2.1", 0) {
            VersionCheck::Mismatch { error, .. } => assert_eq!(error, "version mismatch"),
            _ => panic!("expected mismatch"),
        }
    }

    #[test]
    fn check_version_two_part_lib_keeps_point_nine() {
        // libcob's `lib` starts {9,9,9}; sscanf("3.2") fills only major+minor, leaving point=9. So an
        // app "3.2" (point 0) is app.version < lib.version, and being >= 2.1 it is accepted -- even with
        // a higher patch level than libcob's (the equal-version fast path never fires here). This
        // faithfully mirrors the C, where the {9,9,9} default makes a 2-part PACKAGE_VERSION lenient.
        assert_eq!(
            cob_check_version("prog", "3.2", 1),
            VersionCheck::Ok {
                cannot_check_subscript: false
            }
        );
    }

    #[test]
    fn check_version_newer_app_than_lib_mismatches() {
        // app newer than libcob (4.0 > 3.2.9): not equal, not app < lib -> falls through to mismatch.
        match cob_check_version("prog", "4.0", 0) {
            VersionCheck::Mismatch { .. } => {}
            other => panic!("expected mismatch, got {:?}", other),
        }
    }

    #[test]
    fn is_initialized_tracks_init() {
        let mut rt = ModuleRuntime::new();
        assert!(!rt.cob_is_initialized());
        rt.cob_init(0, &[]);
        assert!(rt.cob_is_initialized());
    }

    #[test]
    fn get_global_ptr_fatals_when_uninit() {
        let rt = ModuleRuntime::new();
        assert_eq!(rt.cob_get_global_ptr(), Err(EnterError::NotInitialized));
    }

    #[test]
    fn extern_init_is_idempotent() {
        let mut rt = ModuleRuntime::new();
        assert_eq!(rt.cob_extern_init(), 0);
        assert!(rt.cob_initialized);
        // second call: no-op.
        assert_eq!(rt.cob_extern_init(), 0);
    }

    #[test]
    fn init_nomain_clears_mainhandle() {
        let mut rt = ModuleRuntime::new();
        rt.cob_init_nomain(2, &[]);
        assert_eq!(rt.check_mainhandle, 0);
        assert!(rt.cob_initialized);
        assert_eq!(rt.cob_argc, 2);
    }

    #[test]
    fn module_enter_first_time_allocates_and_pushes() {
        let mut rt = init_rt();
        rt.cob_argc = 3;
        let mut slot: Option<u64> = None;
        let r = rt.cob_module_global_enter(&mut slot, false, 0, |_| {});
        assert!(matches!(r, Ok((0, _))));
        assert!(slot.is_some());
        assert_eq!(rt.module_stack.len(), 1);
        assert_eq!(rt.module_list.len(), 1);
        // main program: cob_call_params = cob_argc - 1.
        assert_eq!(rt.glob.cob_call_params, 2);
    }

    #[test]
    fn module_enter_uninit_without_autoinit_errors() {
        let mut rt = ModuleRuntime::new();
        let mut slot: Option<u64> = None;
        let r = rt.cob_module_global_enter(&mut slot, false, 0, |_| {});
        assert_eq!(r, Err(EnterError::NotInitialized));
    }

    #[test]
    fn module_enter_uninit_with_autoinit_inits() {
        let mut rt = ModuleRuntime::new();
        let mut slot: Option<u64> = None;
        let r = rt.cob_module_global_enter(&mut slot, true, 0, |_| {});
        assert!(matches!(r, Ok((0, _))));
        assert!(rt.cob_initialized);
    }

    #[test]
    fn recursive_call_without_on_exception_fatals() {
        let mut rt = init_rt();
        let mut slot: Option<u64> = None;
        rt.cob_module_global_enter(&mut slot, false, 0, |_| {}).unwrap();
        // re-enter same slot with entry==0 while still on stack -> recursion.
        let r = rt.cob_module_global_enter(&mut slot, false, 0, |_| {});
        assert_eq!(r, Err(EnterError::Recursive));
    }

    #[test]
    fn recursive_call_with_on_exception_returns_to_caller() {
        let mut rt = init_rt();
        rt.glob.cob_stmt_exception = 1;
        let mut slot: Option<u64> = None;
        rt.cob_module_global_enter(&mut slot, false, 0, |_| {}).unwrap();
        rt.glob.cob_stmt_exception = 1;
        let mut raised = None;
        let r = rt.cob_module_global_enter(&mut slot, false, 0, |id| raised = Some(id));
        assert_eq!(r, Ok((1, slot.unwrap())));
        assert_eq!(raised, Some(COB_EC_PROGRAM_RECURSIVE_CALL));
        assert_eq!(rt.glob.cob_stmt_exception, 0);
    }

    #[test]
    fn module_leave_pops() {
        let mut rt = init_rt();
        let mut slot: Option<u64> = None;
        rt.cob_module_enter(&mut slot, false, |_| {}).unwrap();
        assert_eq!(rt.module_stack.len(), 1);
        assert!(rt.cob_module_leave().is_some());
        assert_eq!(rt.module_stack.len(), 0);
    }

    #[test]
    fn module_free_removes_from_list_and_nulls_slot() {
        let mut rt = init_rt();
        let mut slot: Option<u64> = None;
        rt.cob_module_global_enter(&mut slot, false, 0, |_| {}).unwrap();
        assert_eq!(rt.module_list.len(), 1);
        rt.cob_module_free(&mut slot);
        assert!(slot.is_none());
        assert_eq!(rt.module_list.len(), 0);
    }

    #[test]
    fn module_free_null_slot_is_noop() {
        let mut rt = init_rt();
        let mut slot: Option<u64> = None;
        rt.cob_module_free(&mut slot);
        assert!(slot.is_none());
    }

    #[test]
    fn save_and_restore_func_roundtrip() {
        let mut rt = init_rt();
        let mut slot: Option<u64> = None;
        rt.cob_module_enter(&mut slot, false, |_| {}).unwrap();
        rt.glob.cob_call_params = 5;
        if let Some(top) = rt.module_stack.last_mut() {
            top.module_num_params = 5;
            top.cob_procedure_params_len = 5;
        }
        let fl = rt.cob_save_func(3, 10);
        // numparams = min(3,10) = 3.
        assert_eq!(fl.func_params_len, 3);
        assert_eq!(rt.glob.cob_call_params, 3);
        assert_eq!(rt.module_stack.last().unwrap().cob_procedure_params_len, 3);
        rt.cob_restore_func(&fl);
        assert_eq!(rt.glob.cob_call_params, 5);
        assert_eq!(rt.module_stack.last().unwrap().module_num_params, 5);
        assert_eq!(rt.module_stack.last().unwrap().cob_procedure_params_len, 5);
    }

    #[test]
    fn save_func_clamps_to_eparams() {
        let mut rt = init_rt();
        let mut slot: Option<u64> = None;
        rt.cob_module_enter(&mut slot, false, |_| {}).unwrap();
        let fl = rt.cob_save_func(10, 4);
        assert_eq!(fl.func_params_len, 4); // params > eparams -> eparams.
        assert_eq!(rt.glob.cob_call_params, 4);
    }

    #[test]
    fn parameter_check_ok_when_enough() {
        let mut rt = init_rt();
        rt.glob.cob_call_params = 3;
        assert_eq!(rt.cob_parameter_check("FOO", 2), Ok(()));
        assert_eq!(rt.cob_parameter_check("FOO", 3), Ok(()));
    }

    #[test]
    fn parameter_check_errors_when_too_few() {
        let mut rt = init_rt();
        rt.glob.cob_call_params = 1;
        assert_eq!(
            rt.cob_parameter_check("FOO", 2),
            Err("CALL to FOO requires 2 arguments".to_string())
        );
    }

    #[test]
    fn set_runtime_trace_file_sets_external_flag() {
        let mut rt = init_rt();
        rt.cob_set_runtime_option(CobRuntimeOptionSwitch::TraceFile, Some(7))
            .unwrap();
        assert_eq!(rt.settings.cob_trace_file, Some(7));
        assert_eq!(rt.settings.external_trace_file, 1);
        rt.cob_set_runtime_option(CobRuntimeOptionSwitch::TraceFile, None)
            .unwrap();
        assert_eq!(rt.settings.external_trace_file, 0);
    }

    #[test]
    fn set_runtime_dump_file_null_sets_none_name() {
        let mut rt = init_rt();
        rt.cob_set_runtime_option(CobRuntimeOptionSwitch::DumpFile, None)
            .unwrap();
        assert_eq!(rt.settings.cob_dump_filename.as_deref(), Some("NONE"));
    }

    #[test]
    fn set_runtime_rescan_env_increments() {
        let mut rt = init_rt();
        rt.cob_set_runtime_option(CobRuntimeOptionSwitch::RescanEnv, None)
            .unwrap();
        assert_eq!(rt.settings.rescan_env_requested, 1);
    }

    #[test]
    fn set_runtime_unknown_option_warns() {
        let mut rt = init_rt();
        let r = rt.cob_set_runtime_option_raw(99, None);
        assert!(r.is_err());
    }

    #[test]
    fn get_runtime_punch_file_hidden_when_libcob_owned() {
        let mut rt = init_rt();
        rt.settings.cob_display_punch_file = Some(9);
        rt.settings.cob_display_punch_filename = Some("punch.txt".to_string());
        // libcob owns the filename -> not externalized.
        assert_eq!(
            rt.cob_get_runtime_option(CobRuntimeOptionSwitch::DisplayPunchFile),
            Ok(None)
        );
        // no filename -> externalized.
        rt.settings.cob_display_punch_filename = None;
        assert_eq!(
            rt.cob_get_runtime_option(CobRuntimeOptionSwitch::DisplayPunchFile),
            Ok(Some(9))
        );
    }

    #[test]
    fn get_runtime_rescan_env_is_unknown() {
        let rt = init_rt();
        assert!(rt
            .cob_get_runtime_option(CobRuntimeOptionSwitch::RescanEnv)
            .is_err());
    }

    #[test]
    fn runtime_option_switch_roundtrips_i32() {
        for i in 0..=4 {
            let o = CobRuntimeOptionSwitch::from_i32(i).unwrap();
            assert_eq!(o as i32, i);
        }
        assert!(CobRuntimeOptionSwitch::from_i32(5).is_none());
    }

    #[test]
    fn call_with_exception_check_normal_path() {
        let mut rt = init_rt();
        let r = rt.cob_call_with_exception_check("PROG", 2, 2, 0, |_, _, _| 42);
        assert_eq!(r, 0);
        assert_eq!(rt.exit_code, 42);
    }

    #[test]
    fn call_with_exception_check_longjmp_path() {
        let mut rt = init_rt();
        // non-zero jmp_result models a longjmp back -> returns the code without calling.
        let r = rt.cob_call_with_exception_check("PROG", 0, 0, -2, |_, _, _| {
            panic!("must not call on longjmp")
        });
        assert_eq!(r, -2);
    }

    #[test]
    fn check_version_apply_stamps_gc_version() {
        let mut rt = init_rt();
        let mut slot: Option<u64> = None;
        rt.cob_module_enter(&mut slot, false, |_| {}).unwrap();
        let _ = rt.cob_check_version_apply("prog", "3.2", 0);
        assert_eq!(
            rt.module_stack.last().unwrap().gc_version.as_deref(),
            Some("3.2")
        );
    }

    #[test]
    fn cob_get_settings_ptr_returns_the_settings_block() {
        let mut rt = init_rt();
        // The shared accessor returns a reference to the same settings the mutable accessor mutates.
        let baseline = rt.cob_get_settings_ptr().clone();
        assert_eq!(rt.cob_get_settings_ptr(), &baseline);
        // mutate through the mut accessor, observe it through the shared one.
        *rt.cob_get_settings_ptr_mut() = baseline.clone();
        assert_eq!(rt.cob_get_settings_ptr(), &baseline);
    }
}
