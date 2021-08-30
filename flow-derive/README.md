# flow-derive

### Q & A
1. Proc macro attributes are not yet supported in rust analyzer, and here is a workaround way.
```json
"rust-analyzer.diagnostics.disabled": [
    "no-such-field"
]
```
