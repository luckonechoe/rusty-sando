# Building on Windows (MSVC toolchain)

This host uses the Rust **MSVC** toolchain. `cargo` cannot link unless the
Visual Studio 2022 Build Tools environment is activated first (`link.exe`
must be on PATH along with the Windows SDK and CRT include/lib paths).

## Installed toolchain

- **Visual Studio 2022 Build Tools** v17.14.33
  - Install root: `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`
  - Workload: `Microsoft.VisualStudio.Workload.VCTools` (+ recommended)
  - Windows SDK: `Microsoft.VisualStudio.Component.Windows11SDK.22621`
- **MSVC compiler**: `14.44.35207`
  - Path: `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207`
- **vcvars activation script**:
  `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat`

## Activation pattern (required for every cargo invocation)

Cargo lives in `%USERPROFILE%\.cargo\bin` but is not on the `cmd.exe`
PATH inherited inside `vcvars64`'s subshell, so prepend it explicitly.

### One-liner (PowerShell host -> cmd subshell)

```powershell
cmd /c "`"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat`" && set PATH=%USERPROFILE%\.cargo\bin;%PATH% && cargo check"
```

Replace `cargo check` with `cargo build`, `cargo test`, etc.

### From `cmd.exe` directly

```cmd
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && set PATH=%USERPROFILE%\.cargo\bin;%PATH% && cargo check
```

## Notes for future workers

- **Always** wrap cargo commands with `vcvars64.bat` on this host. Without
  it, cargo fails with `linker 'link.exe' not found` or missing
  `kernel32.lib` / CRT headers.
- Do not rely on a persistent dev shell across tool calls; each shell
  invocation must re-source `vcvars64.bat`.
- Phase 1 (`phase-1-deps`) intentionally produces type errors after the
  link step succeeds; that is the dep-skeleton baseline, not a regression.
