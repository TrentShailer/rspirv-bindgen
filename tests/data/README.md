## Compiling

```console
slangc -target spirv -profile spirv_1_4 tests/data/spec_constants.slang -o tests/data/spv/spec_constants.spv
slangc -target spirv -profile spirv_1_4 tests/data/render_capture.slang -o tests/data/spv/render_capture.spv
slangc -target spirv -profile spirv_1_4 tests/data/render_line.slang -o tests/data/spv/render_line.spv
slangc -target spirv -profile spirv_1_4 tests/data/maximum_reduction.slang -o tests/data/spv/maximum_reduction.spv
slangc -target spirv -profile spirv_1_4 tests/data/push_constants.slang -o tests/data/spv/push_constants.spv
slangc -target spirv -profile spirv_1_4 tests/data/render_selection.slang -o tests/data/spv/render_selection.spv
slangc -target spirv -profile spirv_1_4 tests/data/descriptor_sets.slang -o tests/data/spv/descriptor_sets.spv
```