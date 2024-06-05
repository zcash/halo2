cargo test --release -- test_mycircuit_full_legacy >/dev/null 2>/dev/null
if [ $? -eq 0 ]; then
    echo TEST RELEASE DEFAULT-FEATURES -- OK
else
    echo TEST RELEASE DEFAULT-FEATURES -- FAIL
fi
cargo test --release --all-features -- test_mycircuit_full_legacy >/dev/null 2>/dev/null
if [ $? -eq 0 ]; then
    echo TEST RELEASE ALL-FEATURES -- OK
else
    echo TEST RELEASE ALL-FEATURES -- FAIL
fi
cargo test -- test_mycircuit_full_legacy >/dev/null 2>/dev/null
if [ $? -eq 0 ]; then
    echo TEST DEBUG DEFAULT-FEATURES -- OK
else
    echo TEST DEBUG DEFAULT-FEATURES -- FAIL
fi
cargo test --all-features -- test_mycircuit_full_legacy >/dev/null 2>/dev/null
if [ $? -eq 0 ]; then
    echo TEST DEBUG ALL-FEATURES -- OK
else
    echo TEST DEBUG ALL-FEATURES -- FAIL
fi
cargo llvm-cov --default-features -- test_mycircuit_full_legacy >/dev/null 2>/dev/null
if [ $? -eq 0 ]; then
    echo LLVMCOV DEFAULT-FEATURES -- OK
else
    echo LLVMCOV DEFAULT-FEATURES -- FAIL
fi
cargo llvm-cov --all-features -- test_mycircuit_full_legacy >/dev/null 2>/dev/null
if [ $? -eq 0 ]; then
    echo LLVMCOV ALL-FEATURES -- OK
else
    echo LLVMCOV ALL-FEATURES -- FAIL
fi
