# Wasm Size Benchmarking

This document describes the automated Wasm size benchmarking system implemented for the Stream Scholar contracts to ensure compliance with Soroban's 64KB limit.

## Overview

Soroban smart contracts have a strict size limit of 64KB (65,536 bytes) for the compiled WebAssembly (`.wasm`) file. This automated system ensures that our contracts stay within this limit and provides detailed reporting on size utilization.

## Features

- **Automated CI/CD Integration**: Size checking is automatically performed on every push and pull request
- **Detailed Reporting**: Comprehensive size analysis with remaining capacity and optimization tips
- **Cross-Platform Support**: Works on Linux, macOS, and Windows environments
- **Local Testing**: Scripts available for local development and testing

## CI/CD Pipeline Integration

The size benchmarking is integrated into the GitHub Actions workflow (`.github/workflows/pipeline.yml`) and includes:

1. **Wasm Build**: Compiles the contract for the `wasm32-unknown-unknown` target with release optimizations
2. **Size Analysis**: Calculates the exact file size and compares it against the 64KB limit
3. **Status Reporting**: Provides detailed feedback on size utilization
4. **GitHub Actions Summary**: Creates a formatted report in the Actions summary

### Workflow Steps

```yaml
- name: Install bc for calculations
- name: Build Wasm contract
- name: Check Wasm size
```

## Local Development

### Bash/Unix Systems

Use the provided shell script:

```bash
chmod +x scripts/check-wasm-size.sh
./scripts/check-wasm-size.sh
```

### Windows PowerShell

Use the PowerShell script:

```powershell
.\scripts\check-wasm-size.ps1
```

### Manual Testing

You can also run the commands manually:

```bash
# Build the contract
cargo build --release --target wasm32-unknown-unknown

# Check the size
find target/wasm32-unknown-unknown/release -name "*.wasm" -exec stat -c%s {} \;
```

## Size Limits

| Metric | Value |
|--------|-------|
| Soroban Limit | 64KB (65,536 bytes) |
| Current Target | Stream Scholar contracts |
| Optimization Profile | `release` with size optimizations |

## Optimization Tips

To reduce Wasm size:

1. **Use Release Profile**: Always build with `--release` flag
2. **Review Dependencies**: Remove unused dependencies
3. **Optimize Imports**: Remove unused imports and code
4. **Use Soroban Tools**: Consider `cargo contract optimize` if available
5. **Code Review**: Look for unnecessary code that can be simplified

## Reporting Format

The system provides reports in multiple formats:

### Console Output
```
📁 Wasm file: target/wasm32-unknown-unknown/release/scholar_contracts.wasm
📊 File size: 45678 bytes (44.61 KB)
✅ SUCCESS: Wasm file size (44.61 KB) is within Soroban limit of 64 KB
   Remaining capacity: 19858 bytes (19.39 KB)
```

### GitHub Actions Summary
A formatted table with:
- File information
- Size metrics
- Status indicators
- Optimization tips
- Utilization percentage

## Troubleshooting

### Common Issues

1. **Build Failures**: Ensure Rust and the wasm32-unknown-unknown target are installed
2. **Missing bc Command**: The CI automatically installs `bc`, but local systems may need it
3. **File Not Found**: Ensure the contract builds successfully before size checking

### Error Messages

- **"No .wasm file found!"**: The build failed or produced no output
- **"Size exceeds limit"**: The contract is too large and needs optimization

## Configuration

The size limit and other parameters can be modified in the pipeline:

```bash
LIMIT_BYTES=65536  # 64KB in bytes
LIMIT_KB=64         # 64KB in kilobytes
```

## Future Enhancements

Potential improvements to consider:

1. **Historical Tracking**: Track size changes over time
2. **Size Regression Alerts**: Automatic alerts for size increases
3. **Optimization Suggestions**: AI-powered optimization recommendations
4. **Multi-Contract Support**: Handle multiple contracts in a single repository
5. **Size Budgeting**: Allocate size budgets for different contract features

## Contributing

When contributing to this project:

1. Always test size changes locally
2. Consider the impact on Wasm size
3. Update documentation if limits change
4. Follow the existing code style for the CI scripts
