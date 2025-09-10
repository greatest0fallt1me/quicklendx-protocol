# Fix Commented Out Test Cases

## 🚨 Issue Description
The QuickLendX protocol test suite currently has **18 test cases commented out** due to authorization and type mismatch issues. These tests need to be fixed and re-enabled to ensure comprehensive test coverage.

## 📊 Current Status
- ✅ **33 tests passing** (all active tests work)
- ❌ **18 tests commented out** (need fixes)
- ✅ **Contract compiles and deploys successfully**

## 🐛 Issues to Fix

### 1. Authorization Issues (13 tests)
**Problem**: Tests are failing with "Error(Auth, InvalidAction)" because they call methods requiring authorization without proper setup.

**Affected Tests**:
- `test_audit_statistics`
- `test_audit_query_functionality` 
- `test_audit_trail_creation`
- `test_audit_integrity_validation`
- `test_create_dispute`
- `test_create_dispute_as_investor`
- `test_dispute_under_review`
- `test_dispute_validation`
- `test_duplicate_dispute_prevention`
- `test_get_invoices_with_disputes`
- `test_get_invoices_by_dispute_status`
- `test_resolve_dispute`
- `test_unauthorized_dispute_creation`

**Root Cause**: Methods like `upload_invoice`, `create_dispute`, etc. require proper authorization context in the test environment.

**Example Error**:
```
Error(Auth, InvalidAction)
"Unauthorized function call for address"
```

### 2. Type Mismatch Issues (5 tests)
**Problem**: Tests are failing with "Error(Value, UnexpectedType)" due to parameter type mismatches in contract calls.

**Affected Tests**:
- `test_escrow_creation_on_bid_acceptance`
- `test_escrow_double_operation_prevention`
- `test_escrow_status_tracking`
- `test_escrow_refund`
- `test_escrow_release_on_verification`

**Root Cause**: Contract method calls have incorrect parameter types or missing parameters.

**Example Error**:
```
Error(WasmVm, InvalidAction)
"Error(Value, UnexpectedType)"
```

## 🔧 Technical Details

### Authorization Setup Required
Tests need proper authorization mocking using Soroban's test utilities:

```rust
// Current (failing)
let invoice_id = client.upload_invoice(&business, &amount, &currency, &due_date, &description, &category, &tags);

// Should be (with proper auth setup)
env.mock_auths(&[MockAuth {
    address: &business,
    invoke: &MockAuthInvoke {
        contract: &contract_id,
        fn_name: "upload_invoice",
        args: (&business, &amount, &currency, &due_date, &description, &category, &tags).into_val(&env),
        sub_invokes: &[],
    },
}]);
let invoice_id = client.upload_invoice(&business, &amount, &currency, &due_date, &description, &category, &tags);
```

### Type Mismatch Fixes Needed
Some contract calls may need parameter type corrections or additional required parameters.

## 📁 Files to Update
- `src/test.rs` - Fix commented out test functions

## 🎯 Expected Outcome
After fixing these issues:
- ✅ All 51 tests should pass
- ✅ Comprehensive test coverage restored
- ✅ Authorization flows properly tested
- ✅ Escrow functionality fully tested

## 📋 Tasks for Contributors

### Phase 1: Authorization Fixes
- [ ] Set up proper authorization mocking for `upload_invoice` calls
- [ ] Set up proper authorization mocking for `create_dispute` calls
- [ ] Set up proper authorization mocking for admin functions
- [ ] Test authorization failure scenarios properly
- [ ] Re-enable 13 authorization-related tests

### Phase 2: Type Mismatch Fixes
- [ ] Investigate parameter types in escrow-related method calls
- [ ] Fix any missing or incorrect parameters
- [ ] Verify method signatures match contract implementation
- [ ] Re-enable 5 escrow-related tests

### Phase 3: Verification
- [ ] Run full test suite to ensure all tests pass
- [ ] Verify no regressions in existing passing tests
- [ ] Update test documentation if needed

## 🔍 How to Reproduce

1. Clone the repository
2. Navigate to `quicklendx-contracts/`
3. Run `cargo test`
4. Observe 33 passing tests
5. Uncomment any of the 18 TODO-marked tests
6. Run `cargo test` to see the specific failures

## 📝 Current Test Structure

```rust
// TODO: Fix authorization issues in test environment
// #[test]
fn test_create_dispute() {
    // Test implementation...
}

// TODO: Fix type mismatch issues in escrow tests  
// #[test]
fn test_escrow_creation_on_bid_acceptance() {
    // Test implementation...
}
```

## 🏷️ Labels
- `tests`
- `authorization`
- `type-mismatch`
- `good-first-issue`
- `help-wanted`

## 🔗 Related
- This issue will be linked to the PR that implements the fixes
- Priority: Medium (tests are working, but coverage is incomplete)
- Dependencies: None (can be worked on independently)

## 💡 Notes for Contributors

- The contract itself is working correctly - these are test environment issues
- Focus on one category at a time (authorization vs type mismatches)
- Use existing passing tests as reference for proper patterns
- Consider creating helper functions for common authorization setups
- Test both success and failure scenarios for authorization

## 📚 Resources
- [Soroban Testing Documentation](https://soroban.stellar.org/docs/how-to-guides/testing)
- [Authorization in Soroban](https://soroban.stellar.org/docs/how-to-guides/auth)
- Existing passing tests in the same file for reference patterns