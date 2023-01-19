# Create production graph for items

```
cargo run -- \
    create-production-graph \
        --resolve-deps \
        --items \
            material \
            matrix \
            product \
            component \
            "Conveyor Belt MK.I" \
            "Conveyor Belt MK.II" \
            "Conveyor Belt MK.III" \
        --ignore \
            advanced
```
