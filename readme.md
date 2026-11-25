### Easy animation system focused on code.   
![SCAL logo](./SCAL_logo.svg) 
# Read [Rust Docs](https://docs.rs/scal-core/latest/scal_core/)

# Testing
After Implementing Any New Features Just use:
`cargo test --workspace`
After any bigger changes, visual inspection of all examples might be needed to prevent regression. 

# Package Update Procedure 
0. cargo test + visually inspect all examples if they look right. 
1. update workspace version
2. update scal_core version
3. update flake.nix version
4. git commit and push
5. acargo publish on: core, runtime, ipc

The steps above are automated by `scripts/release.sh <new-version>` (`--dry-run` to preview, `--macros <ver>` to also bump and publish `scal-ipc-macros`).
Check the installed runtime version with `scal-runtime version`.
