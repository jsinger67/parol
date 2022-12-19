# Useful tips

## Build performance

To get an optimized build performance when using `parol`'s
[Builder](https://github.com/jsinger67/parol/blob/main/crates/parol/src/build.rs) API in your
`build.rs` script you can insert the following overrides into your `Cargo.toml` file:

```toml
# Optimized build performance
[profile.dev.build-override]
opt-level = 3

[profile.release.build-override]
opt-level = 3
```

### Credits

Thanks to [dalance](https://github.com/dalance) for reporting
[#49 (build.rs performance)](https://github.com/jsinger67/parol/issues/49)
