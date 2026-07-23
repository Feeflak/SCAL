### Easy animation system focused on code.   
 
# Read [Rust Docs](https://docs.rs/scal-core/latest/scal_core/)

# After Implementing Any New Features
Just use:
`cargo test --workspace`

# Package Update Procedure 
0. visually inspect all examples if they look right. 
1. update workspace version
2. update scal_core version
3. update flake.nix version
4. git push to prod 
5. cargo publish on core runtime and ipc
