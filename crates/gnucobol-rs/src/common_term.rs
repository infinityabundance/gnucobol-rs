//! Port of the **STOP / EXIT / TERMINATE surface** of `libcob/common.c` -- the runtime
//! teardown path behind `STOP RUN`, `GOBACK`, `CBL_ERROR_PROC`/`STOP ERROR`, a hard
//! (internal) failure, and `cob_tidy`.
//!
//! In C these are OS leaves: they ultimately reach `exit(status)`, `raise(SIGABRT)` or a
//! `longjmp` back into `cob_call_with_exception_check`. Per the panel rule, the syscall is
//! the **declared boundary**: this module ports the pure *state transition* (run the exit
//! handlers in priority order, update the recorded exit code) and returns the **action the
//! runtime WOULD take** as a [`TermAction`] enum plus the ordered list of handler ids that
//! were invoked -- which makes the otherwise-irreversible path testable.
//!
//! State that C keeps in file-static globals (`exit_code`, `runtime_err_str`, `exit_hdlrs`,
//! `return_jmp_buffer_set`, `cob_initialized`, `cobsetptr->cob_core_on_error`,
//! `cob_physical_cancel`) is modelled here as the explicit [`ExitState`] struct -- never as
//! Rust statics.
//!
//! ALREADY PORTED elsewhere and NOT re-ported here: the exception family
//! (`cob_add_exception` / `cob_get_last_exception_*`), `cob_check_*`, `cob_sys_*` (including
//! the `cob_sys_exit_proc` registration that builds the handler list -- here the handler list
//! is taken as input). `cob_set_exit_info` is not present in this 3.2 source.

#![forbid(unsafe_code)]

/// One registered exit handler (C: `struct exit_handlerlist { proc; priority; }`).
///
/// The C `proc` is a function pointer returning `int`; we cannot run an arbitrary C function
/// pointer faithfully, so a handler is modelled by an opaque `id` (so callers can assert the
/// invocation order) plus the `int` value its `proc()` would return. The registration order
/// and priority sorting live in the already-ported `cob_sys_exit_proc`; this module consumes
/// the resulting ordered list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExitHandler {
    /// Opaque identity of the handler (so tests can assert the invocation order).
    pub id: i32,
    /// The value the C `proc()` would return (libcob ignores it, but we keep it for fidelity).
    pub proc_ret: i32,
    /// C: `unsigned char priority`.
    pub priority: u8,
}

/// `cob_core_on_error` setting decoded to an enum (C: `cobsetptr->cob_core_on_error`, range
/// `0..=4`, where `4` is the internal "explicit dump creation failed" state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreOnError {
    /// `0`: never dump/abort -- run handlers + terminate, then plain `exit`.
    Never,
    /// `1`: (default-ish) run handlers + terminate, then plain `exit`.
    OnError,
    /// `2`: prevent module unload, run handlers + terminate, then `raise(SIGABRT)`.
    Abort,
    /// `3`: create a core dump file explicitly before terminating.
    Dump,
    /// `4`: internal -- an explicit dump creation failed, so abort instead.
    DumpFailed,
}

impl CoreOnError {
    /// Decode the raw `0..=4` setting; out-of-range values clamp to [`CoreOnError::Never`]
    /// (the C code only ever stores `0..=4`).
    pub fn from_raw(v: u32) -> Self {
        match v {
            1 => CoreOnError::OnError,
            2 => CoreOnError::Abort,
            3 => CoreOnError::Dump,
            4 => CoreOnError::DumpFailed,
            _ => CoreOnError::Never,
        }
    }

    /// Raw `unsigned int` value, matching the C numbering.
    pub fn raw(self) -> u32 {
        match self {
            CoreOnError::Never => 0,
            CoreOnError::OnError => 1,
            CoreOnError::Abort => 2,
            CoreOnError::Dump => 3,
            CoreOnError::DumpFailed => 4,
        }
    }
}

/// The action the runtime WOULD take once the pure state transition is done -- i.e. the OS
/// leaf this module declines to execute. Returned by [`ExitState::cob_stop_run`] and friends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermAction {
    /// C: `longjmp (return_jmp_buf, code)` -- only taken when `return_jmp_buffer_set` and
    /// `COB_WITHOUT_JMP` is not defined. `code` is the longjmp value (`1`, `-1`, `-2`, `-3`).
    Longjmp(i32),
    /// C: `exit (status)`.
    Exit(i32),
    /// C: `raise (SIGABRT)` (signal number `6`).
    RaiseSigabrt,
}

/// Resolution of a teardown call: which OS leaf would be reached, plus the ids of the exit
/// handlers that were invoked in order (the testable byproduct).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TermOutcome {
    /// The OS-level action the runtime would perform.
    pub action: TermAction,
    /// Ids of the exit handlers invoked, in invocation order.
    pub invoked: Vec<i32>,
}

/// `SIGABRT` signal number (POSIX value `6`), for [`TermAction::RaiseSigabrt`] callers.
pub const SIGABRT: i32 = 6;

/// Explicit model of the file-static runtime-exit state of `common.c`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExitState {
    /// C: `static int cob_initialized`.
    pub cob_initialized: bool,
    /// C: `static int exit_code` -- the value returned by [`Self::cob_last_exit_code`].
    pub exit_code: i32,
    /// C: `static char runtime_err_str[...]` -- last runtime error (empty if none).
    pub runtime_err_str: String,
    /// C: ordered handler list `exit_hdlrs` (already priority-sorted by `cob_sys_exit_proc`).
    pub exit_hdlrs: Vec<ExitHandler>,
    /// C: `static int return_jmp_buffer_set` -- whether a `cob_call` is on the stack so a
    /// `longjmp` back to it is possible.
    pub return_jmp_buffer_set: bool,
    /// C: `cobsetptr->cob_core_on_error`.
    pub cob_core_on_error: CoreOnError,
    /// C: `cobsetptr->cob_physical_cancel` -- set to `-1` to prevent module unloading.
    pub cob_physical_cancel: i32,
    /// Models the `COB_CORE_ON_ERROR` environment variable consulted only when the runtime is
    /// **not** initialized (C: `cob_getenv_direct("COB_CORE_ON_ERROR")`). `Some(c)` if the env
    /// var holds a single char `'0'..='3'`.
    pub env_core_on_error: Option<u32>,
    /// Boundary input: whether `create_dumpfile()` would succeed when `core_on_error == 3`.
    /// `true` => dump created (stays `Dump`); `false` => promotes to `DumpFailed`.
    pub dumpfile_succeeds: bool,
}

impl Default for ExitState {
    fn default() -> Self {
        ExitState {
            cob_initialized: true,
            exit_code: 0,
            runtime_err_str: String::new(),
            exit_hdlrs: Vec::new(),
            return_jmp_buffer_set: false,
            cob_core_on_error: CoreOnError::Never,
            cob_physical_cancel: 0,
            env_core_on_error: None,
            dumpfile_succeeds: true,
        }
    }
}

impl ExitState {
    // ---- pure teardown side-effects (modelled, no OS leaf) ---------------------------------

    /// Port of `common.c:cob_exit_common_modules` -- C walks `cob_module_list`, calls each
    /// module's `module_cancel.funcint(-20)` ("clear just decimals"), frees the node, and
    /// finally sets `cob_module_list = NULL`. There is no portable pure state here beyond
    /// "the module list is now empty": the module objects and their cancel callbacks are the
    /// runtime/codegen boundary. Modelled as a no-op state transition that returns the
    /// `-20` cancel argument the runtime would pass to each module.
    pub fn cob_exit_common_modules(&self) -> i32 {
        // C passes -20 to every module_cancel.funcint; the module list itself lives outside
        // this module's modelled state, so this is the faithful pure remainder.
        -20
    }

    /// Port of `common.c:cob_exit_common` -- the final teardown leaf: frees the locale strings,
    /// the cached externals/mallocs, `cobglobptr`/`cobsetptr`, and (the only pure remainder)
    /// sets `cob_initialized = 0`. Everything else is `cob_free` of runtime-owned heap, which
    /// is the declared boundary. Modelled as: mark the runtime uninitialized.
    pub fn cob_exit_common(&mut self) {
        // C: `cob_initialized = 0;` (after freeing all runtime-owned allocations).
        self.cob_initialized = false;
    }

    /// Port of `common.c:ss_terminate_routines` -- the **signal-safe** terminate path. C does:
    /// guard on `cob_initialized && cobglobptr`, flush fileio messages, exit the screen, and
    /// (if a module is active with an abort reason and stacktrace is enabled) write the stack
    /// trace -- but deliberately **no** dump (not signal-safe). No portable pure state mutates
    /// here; modelled as a guarded no-op returning whether the body ran.
    pub fn ss_terminate_routines(&self) -> bool {
        // C: `if (!cob_initialized || !cobglobptr) return;`
        // cobglobptr existence tracks cob_initialized in our model.
        self.cob_initialized
    }

    /// Port of `common.c:cob_terminate_routines` -- the full (non-signal) terminate path: flush
    /// stderr, exit fileio/screen/reportio/mlio/intrinsic/strings/numeric, dump the module if an
    /// abort reason is set, close the trace/dump/punch files, then
    /// `cob_exit_common_modules(); cob_exit_call(); cob_exit_common();`. All of that is file/IO
    /// teardown (the boundary). The faithful pure remainder is the guard plus the final
    /// `cob_exit_common` (which clears `cob_initialized`). Returns whether the body ran.
    pub fn cob_terminate_routines(&mut self) -> bool {
        // C: `if (!cob_initialized || !cobglobptr) return;`
        if !self.cob_initialized {
            return false;
        }
        let _ = self.cob_exit_common_modules();
        self.cob_exit_common();
        true
    }

    /// Port of `common.c:call_exit_handlers_and_terminate` -- walk the `exit_hdlrs` list in
    /// order, call each `proc()` (C resets `cob_source_file/line` and `cob_call_params = 0`
    /// before each), then `cob_terminate_routines()`. Returns the ids of handlers invoked in
    /// order; the per-handler `proc()` return value is ignored by libcob (kept on the struct
    /// for fidelity).
    pub fn call_exit_handlers_and_terminate(&mut self) -> Vec<i32> {
        let mut invoked = Vec::with_capacity(self.exit_hdlrs.len());
        // Clone ids first so we don't borrow `self` across the terminate mutation.
        for h in &self.exit_hdlrs {
            invoked.push(h.id);
        }
        self.cob_terminate_routines();
        invoked
    }

    // ---- the STOP / EXIT / failure surface -------------------------------------------------

    /// Port of `common.c:cob_stop_run` (`STOP RUN`). If the runtime is not initialized C calls
    /// `cob_hard_failure()`; otherwise it runs the exit handlers + terminates, records
    /// `exit_code = status`, and either `longjmp`s back to the enclosing `cob_call` (when
    /// `return_jmp_buffer_set`) or `exit(status)`s.
    pub fn cob_stop_run(&mut self, status: i32) -> TermOutcome {
        if !self.cob_initialized {
            // C: `cob_hard_failure ();` -- which itself returns no value (it exits).
            return self.cob_hard_failure();
        }
        let invoked = self.call_exit_handlers_and_terminate();
        self.exit_code = status;
        let action = if self.return_jmp_buffer_set {
            TermAction::Longjmp(1)
        } else {
            TermAction::Exit(status)
        };
        TermOutcome { action, invoked }
    }

    /// Port of `common.c:cob_stop_error` (`STOP ERROR`). C raises the `"STOP ERROR"` runtime
    /// error then calls `cob_hard_failure()`. We record the runtime error string and delegate.
    pub fn cob_stop_error(&mut self) -> TermOutcome {
        // C: `cob_runtime_error ("STOP ERROR");` -- which sets runtime_err_str.
        self.runtime_err_str = String::from("STOP ERROR");
        self.cob_hard_failure()
    }

    /// Port of `common.c:handle_core_on_error` -- decide the effective `core_on_error` value:
    /// take it from `cobsetptr` when initialized, else from the `COB_CORE_ON_ERROR` env var
    /// (single char `'0'..='3'`, default `0`). If the value is `3` ("explicit core dump"),
    /// attempt `create_dumpfile()`; on failure promote to `4` (and persist `4` into the
    /// setting when initialized). Returns the resolved [`CoreOnError`].
    pub fn handle_core_on_error(&mut self) -> CoreOnError {
        let mut core = if self.cob_initialized {
            self.cob_core_on_error
        } else {
            // C: read COB_CORE_ON_ERROR, accept only a single char '0'..'3'.
            match self.env_core_on_error {
                Some(v @ 0..=3) => CoreOnError::from_raw(v),
                _ => CoreOnError::Never,
            }
        };
        // C: explicit create a coredump file.
        if core == CoreOnError::Dump {
            // create_dumpfile() is the boundary; dumpfile_succeeds models its result.
            if !self.dumpfile_succeeds {
                if self.cob_initialized {
                    self.cob_core_on_error = CoreOnError::DumpFailed;
                }
                core = CoreOnError::DumpFailed;
            }
        }
        core
    }

    /// Shared body of `cob_hard_failure` / `cob_hard_failure_internal`: resolve `core_on_error`,
    /// (unless it is `4`) optionally prevent module unload for `2` then run handlers +
    /// terminate, record `exit_code`, and decide the OS leaf.
    fn hard_failure_body(&mut self, recorded_exit_code: i32) -> TermOutcome {
        let core_on_error = self.handle_core_on_error();
        let mut invoked = Vec::new();
        // C: `if (core_on_error != 4)` -- a failed explicit dump skips the handlers.
        if core_on_error != CoreOnError::DumpFailed {
            if core_on_error == CoreOnError::Abort && self.cob_initialized {
                // C: prevent unloading modules.
                self.cob_physical_cancel = -1;
            }
            invoked = self.call_exit_handlers_and_terminate();
        }
        self.exit_code = recorded_exit_code;
        let action = if self.return_jmp_buffer_set {
            // C: longjmp value is the recorded exit code (-1 for hard_failure, -2 for internal).
            TermAction::Longjmp(recorded_exit_code)
        } else if core_on_error == CoreOnError::Abort || core_on_error == CoreOnError::DumpFailed {
            // C: `cob_raise (SIGABRT);`
            TermAction::RaiseSigabrt
        } else {
            // C: `exit (EXIT_FAILURE);`
            TermAction::Exit(EXIT_FAILURE)
        };
        TermOutcome { action, invoked }
    }

    /// Port of `common.c:cob_hard_failure` -- an unrecoverable runtime abort. Records
    /// `exit_code = -1`. (Provided here as the target of `cob_stop_run`'s uninitialized branch;
    /// the public `cob_hard_failure` itself lives with the rest of the abort surface.)
    pub fn cob_hard_failure(&mut self) -> TermOutcome {
        // C: `exit_code = -1;`
        self.hard_failure_body(-1)
    }

    /// Port of `common.c:cob_hard_failure_internal` -- like [`Self::cob_hard_failure`] but for
    /// an *internal* (port-bug) failure: C prints the `prefix` and `"Please report this!"` to
    /// stderr (the message is the boundary; the `prefix` is accepted for signature fidelity),
    /// resolves `core_on_error`, runs the same teardown, and records `exit_code = -2`.
    pub fn cob_hard_failure_internal(&mut self, _prefix: Option<&str>) -> TermOutcome {
        // The stderr "<prefix>: Please report this!" print is the declared I/O boundary.
        // C: `exit_code = -2;`
        self.hard_failure_body(-2)
    }

    /// Port of `common.c:cob_tidy` -- the `CBL_EXIT` / orderly teardown without `STOP RUN`'s
    /// `exit`. If not initialized C sets `exit_code = -1` and returns `1`; otherwise sets
    /// `exit_code = 0`, runs the exit handlers + terminates, and returns `0`. Returns the C
    /// return value (`0`/`1`) and the invoked handler ids.
    pub fn cob_tidy(&mut self) -> (i32, Vec<i32>) {
        if !self.cob_initialized {
            self.exit_code = -1;
            return (1, Vec::new());
        }
        self.exit_code = 0;
        let invoked = self.call_exit_handlers_and_terminate();
        (0, invoked)
    }

    // ---- getters over the state ------------------------------------------------------------

    /// Port of `common.c:cob_last_exit_code` -- the internal exit code, set on `STOP RUN` /
    /// last `GOBACK` / runtime error / `cob_tidy`.
    pub fn cob_last_exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Port of `common.c:cob_last_runtime_error` -- the last runtime error string (empty if no
    /// runtime error was raised).
    pub fn cob_last_runtime_error(&self) -> &str {
        &self.runtime_err_str
    }
}

/// C: `EXIT_FAILURE` from `<stdlib.h>` (conventionally `1`).
pub const EXIT_FAILURE: i32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    fn hdlrs(ids: &[i32]) -> Vec<ExitHandler> {
        ids.iter()
            .map(|&id| ExitHandler {
                id,
                proc_ret: 0,
                priority: 64,
            })
            .collect()
    }

    #[test]
    fn core_on_error_roundtrip() {
        for v in 0..=4u32 {
            assert_eq!(CoreOnError::from_raw(v).raw(), v);
        }
        // out-of-range clamps to Never.
        assert_eq!(CoreOnError::from_raw(99), CoreOnError::Never);
    }

    #[test]
    fn stop_run_initialized_runs_handlers_and_exits() {
        let mut st = ExitState {
            exit_hdlrs: hdlrs(&[10, 20, 30]),
            ..Default::default()
        };
        let out = st.cob_stop_run(7);
        assert_eq!(out.invoked, vec![10, 20, 30]);
        assert_eq!(out.action, TermAction::Exit(7));
        assert_eq!(st.cob_last_exit_code(), 7);
        // cob_terminate_routines -> cob_exit_common cleared initialized.
        assert!(!st.cob_initialized);
    }

    #[test]
    fn stop_run_longjmps_when_buffer_set() {
        let mut st = ExitState {
            return_jmp_buffer_set: true,
            ..Default::default()
        };
        let out = st.cob_stop_run(3);
        assert_eq!(out.action, TermAction::Longjmp(1));
        assert_eq!(st.cob_last_exit_code(), 3);
    }

    #[test]
    fn stop_run_uninitialized_is_hard_failure() {
        let mut st = ExitState {
            cob_initialized: false,
            ..Default::default()
        };
        let out = st.cob_stop_run(5);
        // hard failure path: exit_code -1, EXIT_FAILURE (core_on_error Never, no jmp).
        assert_eq!(st.cob_last_exit_code(), -1);
        assert_eq!(out.action, TermAction::Exit(EXIT_FAILURE));
    }

    #[test]
    fn stop_error_records_message_and_hard_fails() {
        let mut st = ExitState::default();
        let out = st.cob_stop_error();
        assert_eq!(st.cob_last_runtime_error(), "STOP ERROR");
        assert_eq!(st.cob_last_exit_code(), -1);
        assert_eq!(out.action, TermAction::Exit(EXIT_FAILURE));
    }

    #[test]
    fn handle_core_on_error_from_setting() {
        let mut st = ExitState {
            cob_core_on_error: CoreOnError::Abort,
            ..Default::default()
        };
        assert_eq!(st.handle_core_on_error(), CoreOnError::Abort);
    }

    #[test]
    fn handle_core_on_error_dump_success_stays_dump() {
        let mut st = ExitState {
            cob_core_on_error: CoreOnError::Dump,
            dumpfile_succeeds: true,
            ..Default::default()
        };
        assert_eq!(st.handle_core_on_error(), CoreOnError::Dump);
        assert_eq!(st.cob_core_on_error, CoreOnError::Dump);
    }

    #[test]
    fn handle_core_on_error_dump_failure_promotes_to_four() {
        let mut st = ExitState {
            cob_core_on_error: CoreOnError::Dump,
            dumpfile_succeeds: false,
            ..Default::default()
        };
        assert_eq!(st.handle_core_on_error(), CoreOnError::DumpFailed);
        // persisted into the setting because initialized.
        assert_eq!(st.cob_core_on_error, CoreOnError::DumpFailed);
    }

    #[test]
    fn handle_core_on_error_env_when_uninitialized() {
        let mut st = ExitState {
            cob_initialized: false,
            env_core_on_error: Some(2),
            ..Default::default()
        };
        assert_eq!(st.handle_core_on_error(), CoreOnError::Abort);
        // setting is NOT persisted when uninitialized.
        assert_eq!(st.cob_core_on_error, CoreOnError::Never);
    }

    #[test]
    fn hard_failure_abort_prevents_cancel_and_raises() {
        let mut st = ExitState {
            cob_core_on_error: CoreOnError::Abort,
            exit_hdlrs: hdlrs(&[1, 2]),
            ..Default::default()
        };
        let out = st.cob_hard_failure();
        assert_eq!(out.invoked, vec![1, 2]);
        assert_eq!(st.cob_physical_cancel, -1);
        assert_eq!(out.action, TermAction::RaiseSigabrt);
        assert_eq!(st.cob_last_exit_code(), -1);
    }

    #[test]
    fn hard_failure_dumpfailed_skips_handlers() {
        let mut st = ExitState {
            cob_core_on_error: CoreOnError::Dump,
            dumpfile_succeeds: false,
            exit_hdlrs: hdlrs(&[1, 2]),
            ..Default::default()
        };
        let out = st.cob_hard_failure();
        // core_on_error became 4 => handlers skipped, raise SIGABRT.
        assert!(out.invoked.is_empty());
        assert_eq!(out.action, TermAction::RaiseSigabrt);
    }

    #[test]
    fn hard_failure_internal_records_minus_two() {
        let mut st = ExitState::default();
        let out = st.cob_hard_failure_internal(Some("oops"));
        assert_eq!(st.cob_last_exit_code(), -2);
        assert_eq!(out.action, TermAction::Exit(EXIT_FAILURE));
    }

    #[test]
    fn hard_failure_internal_longjmp_carries_minus_two() {
        let mut st = ExitState {
            return_jmp_buffer_set: true,
            ..Default::default()
        };
        let out = st.cob_hard_failure_internal(None);
        assert_eq!(out.action, TermAction::Longjmp(-2));
    }

    #[test]
    fn tidy_initialized_returns_zero() {
        let mut st = ExitState {
            exit_hdlrs: hdlrs(&[9]),
            ..Default::default()
        };
        let (ret, invoked) = st.cob_tidy();
        assert_eq!(ret, 0);
        assert_eq!(invoked, vec![9]);
        assert_eq!(st.cob_last_exit_code(), 0);
    }

    #[test]
    fn tidy_uninitialized_returns_one() {
        let mut st = ExitState {
            cob_initialized: false,
            ..Default::default()
        };
        let (ret, invoked) = st.cob_tidy();
        assert_eq!(ret, 1);
        assert!(invoked.is_empty());
        assert_eq!(st.cob_last_exit_code(), -1);
    }

    #[test]
    fn terminate_routines_guarded_when_uninitialized() {
        let mut st = ExitState {
            cob_initialized: false,
            ..Default::default()
        };
        assert!(!st.cob_terminate_routines());
        assert!(!st.ss_terminate_routines());
    }

    #[test]
    fn exit_common_clears_initialized() {
        let mut st = ExitState::default();
        st.cob_exit_common();
        assert!(!st.cob_initialized);
        assert_eq!(st.cob_exit_common_modules(), -20);
    }
}
