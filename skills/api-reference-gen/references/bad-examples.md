# API Documentation Anti-Patterns

Reference material for api-reference-gen. These are real patterns to avoid — each includes what's wrong and how to fix it.

## Tautological Documentation

The doc comment restates the function name without adding information.

**Bad:**
```typescript
/**
 * Gets the user.
 * @param id - The id
 * @returns The user
 */
function getUser(id: string): User
```

**Good:**
```typescript
/**
 * Fetches a user record from the database by their unique identifier.
 * Returns null if no user exists with the given ID (does not throw).
 * @param id - UUID of the user account
 * @returns The user record, or null if not found
 */
function getUser(id: string): User | null
```

**Why it's bad:** A developer reading `getUser` already knows it "gets a user." The doc should explain *where* it gets it from, what happens if it's not found, and what format the ID should be.

## Implementation Narration

The doc describes the code line-by-line instead of the observable behaviour.

**Bad:**
```typescript
/**
 * Creates a new Map, iterates over the items array, checks if each item
 * has a valid key using the isValid helper, and if so adds it to the Map.
 * Then converts the Map to an array of entries and returns it.
 */
function indexItems(items: Item[]): [string, Item][]
```

**Good:**
```typescript
/**
 * Indexes an array of items by their key, filtering out items with
 * invalid keys. Duplicate keys keep the last occurrence.
 * @param items - Array of items to index
 * @returns Array of [key, item] entries for valid items only
 */
function indexItems(items: Item[]): [string, Item][]
```

**Why it's bad:** Implementation details change; behaviour contracts are stable. Readers need to know *what* the function guarantees, not *how* it works internally.

## Missing Error Documentation

The function can fail, but the doc doesn't mention how.

**Bad:**
```typescript
/**
 * Saves the configuration to disk.
 * @param config - The configuration object
 */
function saveConfig(config: Config): void
```

**Good:**
```typescript
/**
 * Persists the configuration to `~/.config/app/config.json`.
 * Creates the directory if it doesn't exist.
 * @param config - Configuration object (validated before write)
 * @throws {EACCES} If the config directory is not writable
 * @throws {ValidationError} If config fails schema validation
 */
function saveConfig(config: Config): void
```

**Why it's bad:** Callers need to know what can go wrong to write correct error handling. Undocumented errors become production surprises.

## Parameter Type Without Semantics

The doc lists the type but not what values are valid or what they mean.

**Bad:**
```python
def retry(fn, count, delay):
    """Retry a function.

    Args:
        fn: The function.
        count: The count.
        delay: The delay.
    """
```

**Good:**
```python
def retry(fn, count=3, delay=1.0):
    """Retry a callable on failure with exponential backoff.

    Args:
        fn: Zero-argument callable to retry. Must raise on failure.
        count: Maximum retry attempts (1-10). Default: 3.
        delay: Initial delay in seconds between retries. Doubles
               on each retry. Default: 1.0.
    """
```

**Why it's bad:** Types tell you *what* to pass; semantics tell you *which values* are valid and *how* they're used. Both are needed.

## Outdated Examples

The example uses APIs that no longer exist or have changed signatures.

**Bad:**
```typescript
/**
 * @example
 * // This used to work in v1:
 * const client = new ApiClient('key');
 * const result = client.fetch('/users');  // fetch was renamed to request in v2
 */
```

**How to prevent:**
- Extract examples from current test files (they're verified by CI)
- Reference the example-extraction skill for automated extraction
- Include version annotations if API has changed recently

## Copy-Paste Documentation

Multiple functions share identical documentation that was copy-pasted.

**Bad:**
```go
// ProcessOrder processes the given order.
func ProcessOrder(order Order) error

// ProcessPayment processes the given payment.
func ProcessPayment(payment Payment) error

// ProcessRefund processes the given refund.
func ProcessRefund(refund Refund) error
```

**Good:**
```go
// ProcessOrder validates the order, reserves inventory, and queues
// a fulfillment task. Returns ErrOutOfStock if any item is unavailable.
func ProcessOrder(order Order) error

// ProcessPayment charges the customer's payment method and records
// the transaction. Returns ErrPaymentDeclined if the charge fails.
func ProcessPayment(payment Payment) error

// ProcessRefund reverses a completed payment and restores inventory.
// Can only refund orders in "delivered" status. Returns ErrNotRefundable otherwise.
func ProcessRefund(refund Refund) error
```

**Why it's bad:** Each function has unique behaviour, preconditions, and failure modes. Generic descriptions force developers to read the source code anyway.
